#!/usr/bin/python3
import os
import re
import sys
import yaml
import argparse
import datetime
import subprocess

from virt.qemutest.autoconf import find_qemu_binary

zapterm1 = re.compile(r'\x1b\[[0-9;:=?!]*[HJhlmnpt]')
zapterm2 = re.compile(r'\x1b\][0-9]*')
zapterm3 = re.compile(r'[\x07\r]')

def zapterm(line):
    line = zapterm1.sub('', line)
    line = zapterm2.sub('', line)
    line = zapterm3.sub('', line)
    return line

def now():
    tz = datetime.timezone.utc
    ts = datetime.datetime.now(tz)
    return ts.isoformat()

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
                        type = str, default = '30', metavar = 'SECS',
                        help = 'set timeout (default: 20)')
    parser.add_argument('--snp', dest = 'snp',
                         default = False, action = 'store_true',
                        help = 'boot in sev-snp mode')
    options = parser.parse_args()

    cmdline = [ 'timeout', '--foreground', options.timeout,
                find_qemu_binary(),
                '-enable-kvm',
                '-machine', 'q35,igvm-cfg=fw.igvm',
                '-cpu', 'host',
                '-m', '1G',
                '-nographic', '-no-reboot',
                '-object', f'igvm-cfg,id=fw.igvm,file={options.igvm}' ]

    result = { 'name' : '/boot/in/qemu',
               'start-time' : now() }
    tmtdata = os.environ.get('TMT_TEST_DATA');
    if tmtdata:
        result['log'] = [ 'firmware.log' ]
        cmdline += [ '-chardev', f'file,id=fw.log,path={tmtdata}/firmware.log' ]
        cmdline += [ '-device', 'isa-debugcon,iobase=0x402,chardev=fw.log' ]

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
        if tmtdata:
            with open(f'{tmtdata}/stderr.log', 'w') as l:
                l.write(rc.stderr.decode())
            result['log'].append('stderr.log')
        else:
            print('=== stderr ===')
            print(zapterm(rc.stderr.decode()))

    if len(rc.stdout):
        if tmtdata:
            with open(f'{tmtdata}/stdout.log', 'w') as l:
                l.write(rc.stdout.decode())
            result['log'].append('stdout.log')
        else:
            print('=== stdout ===')
            print(zapterm(rc.stdout.decode()))

    if not options.expect in rc.stdout.decode():
        exitcode = 1
        result['result'] = 'fail'
        print(f'=== FAIL ("{options.expect}" not found) ===')
    else:
        exitcode = 0
        result['result'] = 'pass'
        print('=== PASS ===')

    if tmtdata:
        result['end-time'] = now()
        results = [ result, ]
        y = yaml.dump(results)
        with open(f'{tmtdata}/tmt-report-results.yaml', 'a') as r:
            r.write(y)

    return exitcode

if __name__ == '__main__':
    sys.exit(main())
