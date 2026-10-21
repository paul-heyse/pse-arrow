import sys; sys.path.insert(0,'.')
from race_lib import scenario
from q import run
print(run("DEFINE FIELD touch ON studies TYPE int DEFAULT 0;", "q3"))
scenario("E2: deleter first (delete+sleep); ref-creator increments target.touch then creates ref, commits first",
    "CREATE studies:c9;",
    "BEGIN; DELETE studies:c9; sleep(2s); COMMIT;",
    "BEGIN; UPDATE studies:c9 SET touch += 1; CREATE obligations:r9 SET owner=studies:c9; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r9; SELECT * FROM studies:c9;")
scenario("F2: ref-creator increments target.touch, creates ref, sleeps; deleter commits during sleep",
    "CREATE studies:c10;",
    "BEGIN; UPDATE studies:c10 SET touch += 1; CREATE obligations:r10 SET owner=studies:c10; sleep(2s); COMMIT;",
    "BEGIN; DELETE studies:c10; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r10; SELECT * FROM studies:c10;")
scenario("N: no-op UPDATE (SET retired = retired) vs concurrent UPDATE of another field: is the no-op written?",
    "CREATE studies:c11;",
    "BEGIN; UPDATE studies:c11 SET retired = retired; sleep(2s); COMMIT;",
    "BEGIN; UPDATE studies:c11 SET touch = 5; COMMIT;",
    check="SELECT * FROM studies:c11;")
