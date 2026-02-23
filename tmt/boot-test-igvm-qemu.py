#!/usr/bin/python3
import os
import sys
import argparse
import subprocess

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--igvm', dest = 'igvm',
                        type = str, required = True,
                        help = 'set firmware image (required)')
    parser.add_argument('--expect', dest = 'expect',
                        type = str, required = True,
                        help = 'set expected guest output (required)')
    parser.add_argument('--timeout', dest = 'timeout',
                        type = str, default = '10',
                        help = 'set timeout (default: 10)')
    parser.add_argument('--kernel', dest = 'kernel',
                        type = str,
                        help = 'set kernel image (optional)')
    parser.add_argument('--append', dest = 'append',
                        type = str,
                        help = 'set kernel command line (optional)')
    options = parser.parse_args()

    cmdline = [ 'timeout', '--foreground', options.timeout,
                'qemu-system-x86_64',
                '-enable-kvm',
                '-machine', 'q35,igvm-cfg=fw.igvm',
                '-m', '1G',
                '-nographic', '-no-reboot',
                '-object', f'igvm-cfg,id=fw.igvm,file={options.igvm}' ]
    if options.kernel:
        cmdline += [ '-kernel', options.kernel ]
        if options.append:
            cmdline += [ '-append', options.append ]
    print(cmdline)

    rc = subprocess.run(cmdline, capture_output = True, check = False)
    if len(rc.stderr):
        print('=== stderr ===')
        print(rc.stderr.decode())
    if len(rc.stdout):
        print('=== stdout ===')
        print(rc.stdout.decode())

    if not options.expect in rc.stdout.decode():
        print(f'=== FAIL ("{options.expect}" not found) ===')
        return 1
    else:
        print('=== PASS ===')
        return 0

if __name__ == '__main__':
    sys.exit(main())
