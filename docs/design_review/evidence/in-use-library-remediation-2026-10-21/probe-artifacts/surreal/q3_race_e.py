import sys; sys.path.insert(0,'.')

from race_lib import scenario
scenario("E: delete+sleep first; ref-creator touches target (write) then creates ref, commits first",
    "CREATE studies:c6;",
    "BEGIN; DELETE studies:c6; sleep(2s); COMMIT;",
    "BEGIN; UPDATE studies:c6 SET retired = retired; CREATE obligations:r6 SET owner=studies:c6; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r6; SELECT * FROM studies:c6;")
scenario("F: ref-creator touches target then sleeps; deleter runs during sleep",
    "CREATE studies:c7;",
    "BEGIN; UPDATE studies:c7 SET retired = retired; CREATE obligations:r7 SET owner=studies:c7; sleep(2s); COMMIT;",
    "BEGIN; DELETE studies:c7; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r7; SELECT * FROM studies:c7;")
scenario("G: deleter reads target FOR UPDATE + delete + sleep; ref-creator reads FOR UPDATE + creates, commits first",
    "CREATE studies:c8;",
    "BEGIN; SELECT * FROM studies:c8 FOR UPDATE; DELETE studies:c8; sleep(2s); COMMIT;",
    "BEGIN; SELECT * FROM studies:c8 FOR UPDATE; CREATE obligations:r8 SET owner=studies:c8; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r8; SELECT * FROM studies:c8;")
