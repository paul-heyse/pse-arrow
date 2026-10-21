import sys; sys.path.insert(0,'.')
from race_lib import scenario
scenario("W1: write-write same record (UPDATE vs UPDATE)",
    "UPSERT kk:1 SET v=0;",
    "BEGIN; UPDATE kk:1 SET v=1; sleep(2s); COMMIT;",
    "BEGIN; UPDATE kk:1 SET v=2; COMMIT;",
    check="SELECT * FROM kk:1;")
scenario("W2: read-modify-write v+=1 both",
    "UPSERT kk:2 SET v=0;",
    "BEGIN; UPDATE kk:2 SET v+=1; sleep(2s); COMMIT;",
    "BEGIN; UPDATE kk:2 SET v+=1; COMMIT;",
    check="SELECT * FROM kk:2;")
scenario("W3: FOR UPDATE read then write in T1, T2 writes",
    "UPSERT kk:3 SET v=0;",
    "BEGIN; SELECT * FROM kk:3 FOR UPDATE; UPDATE kk:3 SET v+=1; sleep(2s); COMMIT;",
    "BEGIN; UPDATE kk:3 SET v+=1; COMMIT;",
    check="SELECT * FROM kk:3;")
