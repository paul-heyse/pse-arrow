import re,sys,collections
p=sys.argv[1]
lines=open(p).read().split('\n')
print(f"=== {p} : {len(lines)} lines ===")
# marker census (exclude the lane-table templates and the rule statement)
text='\n'.join(lines)
for m in ['[ref:','[c7:','[docs.rs:','[rustdoc:','[probe]','[gh:','[crates.io:','[bp:','[UNVERIFIED]','[api:']:
    n=text.count(m)
    if n: print(f"  {m:<14} {n}")
# table well-formedness: for each contiguous table block, all rows same pipe count
def pipes(l):
    # ignore pipes inside backtick spans and backslash-escaped pipes
    out=[];inb=False;prev=''
    for ch in l:
        if ch=='`': inb=not inb
        elif ch=='|' and not inb and prev!='\\': out.append(ch)
        prev=ch
    return len(out)
bad=[];block=[];start=0
for i,l in enumerate(lines+['']):
    if l.strip().startswith('|'):
        if not block: start=i+1
        block.append((i+1,pipes(l)))
    else:
        if len(block)>1:
            counts=collections.Counter(c for _,c in block)
            if len(counts)>1:
                mode=counts.most_common(1)[0][0]
                for ln,c in block:
                    if c!=mode: bad.append((ln,c,mode))
        block=[]
print(f"  malformed table rows: {len(bad)}")
for ln,c,mode in bad[:12]: print(f"    line {ln}: {c} pipes, expected {mode}  ->  {lines[ln-1][:110]}")
# section sequence
secs=[(i+1,l) for i,l in enumerate(lines) if l.startswith('## ')]
nums=[]
for ln,l in secs:
    m=re.match(r'## (\d+)\.',l)
    if m: nums.append(int(m.group(1)))
print(f"  sections: {nums}  contiguous={nums==list(range(0,len(nums)))}")
