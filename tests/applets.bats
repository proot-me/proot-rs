#!/usr/bin/env bats

load helper


function script_test_run_sh {
    echo $(/bin/ls -la | /bin/wc -l)
}


@test "test proot-rs run /bin/sh" {
    proot-rs --rootfs "$ROOTFS" -- /bin/sh -e -c "$(declare -f script_test_run_sh); script_test_run_sh"
}


function script_test_run_applets_file_ops {
    PATH=/bin

    # pwd
    [ "$(/bin/pwd -L)" = "/" ]
    cd /tmp/test_applets_file_ops/
    [ "$(/bin/pwd -L)" = "/tmp/test_applets_file_ops" ]

    # ls
    [ "$(ls)" = $'dir1\nfile1' ]

    # touch
    /bin/touch file2
    [ "$(ls)" = $'dir1\nfile1\nfile2' ]

    # cp
    /bin/echo "test" > file2
    cp file2 file3
    [ "$(ls)" = $'dir1\nfile1\nfile2\nfile3' ]

    # mv
    mv file3 dir1/file4
    [ "$(ls)" = $'dir1\nfile1\nfile2' ]

    # find
    [ "$(find .)" = $'.\n./dir1\n./dir1/file4\n./file1\n./file2' ]

    # cat
    [ "$(cat file2)" = "test" ]
    [ "$(cat dir1/file4)" = "test" ]

    # rm
    rm dir1/file4
    [ ! -f dir1/file4  ]

    # ln
    ln -s non_exist_file link1
    [ "$(readlink link1)" = "non_exist_file" ]

    # mkdir
    [[ "$(mkdir dir2/dir3/dir4 2>&1)" == *"No such file or directory"* ]]
    mkdir -p dir2/dir3/dir4
    [[ "$(find .)" == *"./dir2/dir3/dir4"* ]]

    # rmdir
    [[ "$(rmdir dir2 2>&1)" == *"Directory not empty"* ]]
    rmdir dir2/dir3/dir4
    rmdir dir2/dir3
    rmdir dir2

    # mktemp
    local tmp_file="$(mktemp)"
    [[ "$(stat ${tmp_file})" == *"regular empty file"* ]]
    rm "${tmp_file}"

    # stat
    [[ "$(stat dir1)" == *"directory"* ]]
    [[ "$(stat file1)" == *"regular file"* ]]

    # chattr
    local output="$(chattr +i file2 2>&1)"
    local status="$?"
    [[ "$output" == *"Inappropriate ioctl for device"* ]] || [[ "$output" == *"Operation not permitted"* ]] || ( [ "$output" = "" ] && [ "$status" -eq 0 ] && chattr -i file2 )

    # mknod
    mknod test_mknod p
    [[ "$(stat test_mknod)" == *"fifo"* ]]

    # chmod
    touch file2
    chmod 700 file2
    [[ "$(stat file2)" == *"-rwx------"* ]]

    # chown
    # Only users with the `CAP_CHOWN` capability can change the owner of a file.
    local output="$(chown 65535 file2 2>&1)"
    [ "$?" -eq 0 ] || [[ "$output" = *"Operation not permitted"* ]]

}


@test "test proot-rs run applets file operations" {
    local test_dir="$ROOTFS/tmp/test_applets_file_ops"
    mkdir -p "$test_dir"
    mkdir -p "$test_dir/dir1"
    echo "this is file1" > "$test_dir/file1"

    runp proot-rs --rootfs "$ROOTFS" -- /bin/sh -e -x -c "$(declare -f script_test_run_applets_file_ops); script_test_run_applets_file_ops"
    rm -rf "$test_dir"
    [ "$status" -eq 0 ]
}


function script_test_run_applets_common_tools {
    PATH=/bin

    # clear
    clear

    # reset
    reset

    # false
    ! /bin/false

    # true
    /bin/true

    # yes
    # head
    [ "$(yes 2>&- | head -n 3)" = $'y\ny\ny' ]

    # tail
    [ "$(/bin/echo -e '1\n2\n3' | tail -n 1)" = "3" ]

    # echo
    # wc
    [ "$(/bin/echo -e '1\n2\n3' | wc -l)" == 3 ]

    # tee
    [ "$(echo 'test tee' | tee file1)" = "test tee" ]
    [ "$(cat file1)" = "test tee" ]

    # du
    [ "$(du -h file1)" = "4.0K    file1" ]

    # base64
    [ "$(echo 'base64' | base64)" = "YmFzZTY0Cg==" ]

    # md5sum
    [ "$(echo 'md5sum' | md5sum)" = "e65b0dce58cbecf21e7c9ff7318f3b57  -" ]

    # sha256sum
    [ "$(echo 'sha256sum' | sha256sum)" = "5e913e218e1f3fcac8487d7fbb954bd9669f72a7ef6e9d9f519d94b6a8cc88b9  -" ]

    # sha512sum
    [ "$(echo 'sha512sum' | sha512sum)" = "0749dddfaecad1445f66f118738cdb8917dd6adc7f7589c9d30e1d243c541f3b22cf5a77c5bbf6a70a4d078170427439b6e236249815e1281da238eefb5ec1d7  -" ]

    # sed
    [ "$(echo 'hello world' | sed 's/world/sed/g')" = "hello sed" ]

    # sort
    # uniq
    [ "$(echo -e '1\n6\n7\n8\n3\n5\n6\n3\n2\n1\n3\n3\n3\n2\n1' | sort | uniq)" = "$(echo -e '1\n2\n3\n5\n6\n7\n8')" ]

    # grep
    [ "$(echo -e 'hello\nworld' | grep 'world')" = "world" ]

    # awk
    [ "$(echo 'hello' | awk '{print $1 " world" }')" = "hello world" ]

    # bc
    [ "$(echo "10^2" | bc)" = "100" ]

    # strings
    /bin/strings /bin/sh | grep "/bin/sh"

    # split
    echo "0123456789" | split -b 4 - test_split.
    [ "$(ls test_split.*)" = $'test_split.aa\ntest_split.ab\ntest_split.ac' ]

    # date
    [ "$(date -d @0)" = "Thu Jan  1 00:00:00 UTC 1970" ]

    # less
    [ "$(less file1)" = "test tee" ]

    # tr
    [ "$(echo "hello WORLD" | tr "[:upper:]" "[:lower:]")" = "hello world" ]

    # args
    [ "$(echo "echo 'hello world'" | xargs)" = "hello world" ]

    # which
    [ "$(which sh)" = "/bin/sh" ]

    # which
    [ "$(printf '%s world' hello)" = "hello world" ]
    [ "$(printf '%x\n' 3735928559)" = "deadbeef" ]

    # sleep
    sleep 0.001

    # ar
    ar -r archive.a file1
    [ "$(ar -t archive.a)" = "file1" ]
    rm archive.a

    # bzip2
    # bunzip2
    bzip2 file1
    bunzip2 file1.bz2
    [ -f file1 ]

    # gzip
    # gunzip
    gzip file1
    gunzip file1.gz
    [ -f file1 ]

    # tar
    tar cf archive.tar file1
    [ "$(tar tf archive.tar)" = "file1" ]
    tar xf archive.tar
    [ -f file1 ]

    # unzip
    echo "UEsDBC0AAAAAAB1G8VItOwiv//////////8BABQALQEAEAAMAAAAAAAAAAwAAAAAAAAAaGVsbG8gd29ybGQKUEsBAh4DLQAAAAAAHUbxUi07CK8MAAAADAAAAAEAAAAAAAAAAQAAAIARAAAAAC1QSwYGLAAAAAAAAAAeAy0AAAAAAAAAAAABAAAAAAAAAAEAAAAAAAAALwAAAAAAAAA/AAAAAAAAAFBLBgcAAAAAbgAAAAAAAAABAAAAUEsFBgAAAAABAAEALwAAAD8AAAAAAA==" | base64 -d > archive.zip
    [ "$(unzip -p archive.zip)" = "hello world" ]

    # dd
    dd if=file1 of=file2
    [ -f file1 ]
    [ -f file2 ]
    diff file1 file2

    # sh
    [ "$(sh -c "echo hello world")" = "hello world" ]

    # ash
    [ "$(ash -c "echo hello world")" = "hello world" ]

}


@test "test proot-rs run applets common tools" {
    local test_dir="$ROOTFS/tmp/test_applets_common_tools"
    mkdir "$test_dir"
    runp proot-rs --rootfs "$ROOTFS" --cwd "/tmp/test_applets_common_tools" -- /bin/sh -e -x -c "$(declare -f script_test_run_applets_common_tools); script_test_run_applets_common_tools"
    rm -rf "$test_dir"
    [ "$status" -eq 0 ]
}


@test "test proot-rs run uname" {
    local output_on_host="$(uname)"
    runp proot-rs --rootfs "$ROOTFS" -- /bin/uname
    [ "$status" -eq 0 ]
    local output_on_guest="$output"

    [ "$(output_on_host)" = "$(output_on_guest)" ]
}


@test "test proot-rs run whoami" {
    local output_on_host="$(whoami)"
    runp proot-rs -- /bin/whoami
    [ "$status" -eq 0 ]
    local output_on_guest="$output"

    [ "$(output_on_host)" = "$(output_on_guest)" ]
}


@test "test proot-rs run man " {
    local output_on_host="$(man man)"
    runp proot-rs -- /bin/man man
    [ "$status" -eq 0 ]
    local output_on_guest="$output"

    [ "$(output_on_host)" = "$(output_on_guest)" ]
}


@test "test proot-rs run ps" {
    if [ ! -d "/proc" ]; then
        skip "/proc not found in host"
    fi
    runp proot-rs -- /bin/ps -o pid,ppid,user
    [ "$status" -eq 0 ]
    echo "${lines[0]}" | grep 'PID\s*PPID\s*USER'
}

@test "test proot-rs run kill" {

    runp proot-rs --rootfs "$ROOTFS" -- /bin/sh -e -x -c '
        PATH=/bin
        sleep 10 &
        pid="$!"
        # kill the background process
        kill "$pid"
        # check if the process is really dead
        ! kill -0 "$pid"
    '
    [ "$status" -eq 0 ]
}


@test "test proot-rs run pkill" {
    if [ ! -d "/proc" ]; then
        skip "/proc not found in host"
    fi

    runp proot-rs -- /bin/sh -e -x -c '
        PATH=/bin
        sleep 10 &
        pid="$!"
        # kill the background process
        pkill sleep
        # check if the process is really dead
        ! kill -0 "$pid"
    '
    [ "$status" -eq 0 ]
}



@test "test proot-rs run ping" {
    # ping localhost once
    proot-rs -- /bin/ping -c 1 127.0.0.1
}


@test "test proot-rs run wget" {
    proot-rs --rootfs "$ROOTFS" -- /bin/wget http://1.1.1.1 -O -
}

