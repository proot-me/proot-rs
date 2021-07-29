#!/usr/bin/env bats

load ../helper


@test "test execute dynamically linked binary" {
    # compile test case
    compile_c_dynamic "$ROOTFS/bin/test_execve_dynamic" "$BATS_TEST_DIRNAME/test_execve.c"
    runp proot-rs --rootfs "$ROOTFS" --cwd / -- /bin/test_execve_dynamic
    # remember to delete the binary file
    rm "$ROOTFS/bin/test_execve_dynamic"
    [ "$status" -eq 0 ]
}


@test "test execute statically linked binary" {
    # compile test case
    compile_c_static "$ROOTFS/bin/test_execve_static" "$BATS_TEST_DIRNAME/test_execve.c"
    runp proot-rs --rootfs "$ROOTFS" --cwd / -- /bin/test_execve_static
    # remember to delete the binary file
    rm "$ROOTFS/bin/test_execve_static"
    [ "$status" -eq 0 ]
}


function script_test_run_script_with_shebang {
    PATH=/bin

    cd /tmp/test_run_script_with_shebang

    # Test normal shebang script file
    echo '#!/bin/echo 123' > ./script1.sh
    chmod +x ./script1.sh
    [ "$(./script1.sh --something 2>&1)" = "123 ./script1.sh --something" ]

    # Test bad interpreter file
    echo '#!nonexist' > ./script2.sh
    chmod +x ./script2.sh
    [[ "$(./script2.sh 2>&1)" == *"not found"* ]]

    # Test interpreter with relative path
    echo '#!echo' > ./script3.sh
    chmod +x ./script3.sh
    [[ "$(./script3.sh 2>&1)" == *"not found"* ]]

    cd /bin/
    [ "$(/tmp/test_run_script_with_shebang/script3.sh 2>&1)" = "/tmp/test_run_script_with_shebang/script3.sh" ]
    cd -

    # Test optional argument
    echo '#!/bin/echo "123"' > ./script4.sh
    chmod +x ./script4.sh
    [ "$(./script4.sh 2>&1)" = '"123" ./script4.sh' ]

    echo '#!/bin/echo "123      "' > ./script5.sh
    chmod +x ./script5.sh
    [ "$(./script5.sh 2>&1)" = '"123      " ./script5.sh' ]

    # Blank-space in the middle of opt argument is reserved. But the ones at the beginning and the end are stripped
    echo '#!/bin/echo     123 456    789   ' > ./script6.sh
    chmod +x ./script6.sh
    [ "$(./script6.sh 2>&1)" = '123 456    789 ./script6.sh' ]

    # Test blank-space after shebang
    echo '#!   /bin/echo    ' > ./script7.sh
    chmod +x ./script7.sh
    [ "$(./script7.sh 2>&1)" = './script7.sh' ]

    # Test '\0' in shebang line
    echo -e '#!\0/bin/echo' > ./script8.sh
    chmod +x ./script8.sh
    [[ "$(./script8.sh 2>&1)" == *"Permission denied"* ]] # EACCES

    echo -e '#!/bin/echo\0 123' > ./script9.sh
    chmod +x ./script9.sh
    [ "$(./script9.sh 2>&1)" = './script9.sh' ]

    echo -e '#!/bin/echo 123   \0   ' > ./script10.sh
    chmod +x ./script10.sh
    [ "$(./script10.sh 2>&1)" = '123    ./script10.sh' ]

    # shebang length exceed 256
    echo '#!../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../bin/echo 123' > ./script11.sh
    chmod +x ./script11.sh
    ! [[ "$(./script11.sh 2>&1)" == *'123 ./script11.sh'* ]]

}

@test "test run script with shebang" {
    local test_dir="$ROOTFS/tmp/test_run_script_with_shebang"
    mkdir "$test_dir"
    runp proot-rs --rootfs "$ROOTFS" -- /bin/sh -e -x -c "$(declare -f script_test_run_script_with_shebang); script_test_run_script_with_shebang"
    rm -rf "$test_dir"
    [ "$status" -eq 0 ]
}