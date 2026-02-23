#!/usr/bin/python3
import os
import sys
import argparse
import subprocess

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--igvm', dest = 'igvm',
                        type = str, required = True, metavar = 'FIRMWARE',
                        help = 'set firmware image (required)')
    parser.add_argument('--expect', dest = 'expect',
                        type = str, required = True, metavar = 'MATCH',
                        help = 'set expected guest output (required)')
    parser.add_argument('--kernel', dest = 'kernel',
                        type = str, metavar = 'EFI',
                        help = 'set kernel image (optional)')
    parser.add_argument('--append', dest = 'append',
                        type = str, metavar = 'ARGS',
                        help = 'set kernel command line (optional)')
    parser.add_argument('--timeout', dest = 'timeout',
                        type = str, default = '10', metavar = 'SECS',
                        help = 'set timeout (default: 10)')
    parser.add_argument('--snp', dest = 'snp',
                         default = False, action = 'store_true',
                        help = 'boot in sev-snp mode')
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
    if options.snp:
        cmdline += [ '-object', 'sev-snp-guest,id=snp0,reduced-phys-bits=1,cbitpos=51' ]
        cmdline += [ '-machine', 'confidential-guest-support=snp0' ]
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
