#!/usr/bin/env python

import subprocess
import argparse
import re

def main():
    parser = argparse.ArgumentParser(description='Args')

    parser.add_argument(
        'conf',
        help='openvpn conf file'
    )

    parser.add_argument(
        '-R',
        '--remove-route',
        action='store_const',
        const=True,
        default=False,
        help='remove route'
    )

    args = parser.parse_args()

    ps = subprocess.run(['ps', '-A'], capture_output=True)
    if re.search('openvpn', str(ps.stdout)) is not None:
        print('openvpn is running')
        return

    default_net_addr = '128.129.0'
    is_default_net = False
    while not is_default_net:
        openvpn = subprocess.Popen(['openvpn', args.conf], stdout=subprocess.PIPE)
        route_list = []
        try:
            while openvpn.poll() == None:
                line = str(openvpn.stdout.readline())

                # match net addr
                m = re.findall(r'net_addr_v4_add: ([^ ]+)', line)
                if len(m) != 0:
                    print('openvpn use ip: ', m[0])
                    if default_net_addr not in m[0]:
                        print('openvpn kill for use default ip: ', m[0])
                        openvpn.kill()
                        break
                    else:
                        is_default_net = True

                # match net route
                m = re.findall(r'net_route_v4_add: ([^ ]+)', line)
                if len(m) != 0:
                    print('openvpn add route: ', m[0])
                    route_list.append(m[0])

                # match end
                m = re.search(r'Initialization Sequence Completed', line)
                if m is not None:
                    break

            if is_default_net and args.remove_route:
                for route in route_list:
                    print('openvpn del route: ', route)
                    subprocess.run(['ip', 'route', 'del', route, 'via', '128.129.0.1'])
        except Exception:
            openvpn.kill()

    
if __name__ == '__main__':
    main()
