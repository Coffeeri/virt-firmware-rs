#!/usr/bin/python3
import os
import re
import sys
import argparse
import subprocess

zapterm1 = re.compile(r'\x1b\[[0-9;:=?!]*[HJhlmnpt]')
zapterm2 = re.compile(r'\x1b\][0-9]*')
zapterm3 = re.compile(r'[\x07\r]')

def zapterm(line):
    line = zapterm1.sub('', line)
    line = zapterm2.sub('', line)
    line = zapterm3.sub('', line)
    return line

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
                        type = str, default = '20', metavar = 'SECS',
                        help = 'set timeout (default: 20)')
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
                '-object', f'igvm-cfg,id=fw.igvm,file={options.igvm}',
                '-chardev', 'file,id=fw.log,path=firmware.log',
                '-device', 'isa-debugcon,iobase=0x402,chardev=fw.log' ]
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
        print(zapterm(rc.stderr.decode()))
    if len(rc.stdout):
        print('=== stdout ===')
        print(zapterm(rc.stdout.decode()))

    if not options.expect in rc.stdout.decode():
        with open('firmware.log') as f:
            print('=== firmware.log (tail) ===')
            lines = f.read().splitlines();
            for l in lines[ -30 : ]:
                print(l.strip())
        print(f'=== FAIL ("{options.expect}" not found) ===')
        return 1
    else:
        print('=== PASS ===')
        return 0

if __name__ == '__main__':
    sys.exit(main())
