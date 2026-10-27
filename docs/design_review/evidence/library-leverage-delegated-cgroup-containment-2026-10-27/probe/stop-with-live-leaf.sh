#!/usr/bin/env bash
# Stop a delegated scope whose driver holds a populated leaf (child + setsid
# grandchild + double-forked daemon); report survivors and cgroup removal.
set -u
here=$(cd "$(dirname "$0")" && pwd)
unit=pse-probe-delegate-4
state=${1:?state file}
rm -f "$state"
systemd-run --user --scope --quiet --collect --unit=$unit -p Delegate=yes \
  -p MemoryMax=256M -p MemorySwapMax=0 -p TasksMax=64 -- \
  /usr/bin/python3 -I "$here/driver.py" hold "$state" &
runner=$!
for _ in $(seq 100); do [ -s "$state" ] && break; sleep 0.1; done
echo "state: $(cat "$state")"
scope=$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["scope"])' "$state")
leaf=$scope/leaf-hold
pids=$(python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));print(d["driver"],*d["members"].values())' "$state")
echo "unit Delegate: $(systemctl --user show -p Delegate -p DelegateControllers -p KillMode $unit.scope | tr '\n' ' ')"
echo "scope xattrs (outside): $(python3 -c 'import os,sys;p=sys.argv[1];print({n:os.getxattr(p,n) for n in os.listxattr(p)})' "$scope")"
echo "scope subtree_control: [$(cat $scope/cgroup.subtree_control)] leaf procs: $(cat $leaf/cgroup.procs | tr '\n' ' ')"
echo "leaf events: $(cat $leaf/cgroup.events | tr '\n' ' ')"
t0=$(date +%s.%N)
systemctl --user stop $unit.scope; echo "stop rc=$?"
t1=$(date +%s.%N)
echo "stop seconds: $(python3 -c "print(round($t1-$t0,3))")"
wait $runner; echo "systemd-run rc=$?"
for p in $pids; do
  if [ -e /proc/$p ]; then echo "pid $p ALIVE: $(grep State /proc/$p/status)"; else echo "pid $p gone"; fi
done
[ -e "$leaf" ] && echo "leaf still exists" || echo "leaf removed"
[ -e "$scope" ] && echo "scope cgroup still exists" || echo "scope cgroup removed"
systemctl --user status $unit.scope --no-pager 2>&1 | head -3
journalctl --user -u $unit.scope --no-pager -o short-monotonic 2>/dev/null | tail -5
