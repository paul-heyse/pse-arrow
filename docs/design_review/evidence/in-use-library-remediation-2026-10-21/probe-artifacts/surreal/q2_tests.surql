fn::p::top('obj');
fn::p::top('str');
fn::p::inif('obj', true);
fn::p::inif('str', true);
fn::p::infor('obj');
fn::p::infor('str');
fn::p::nested('obj');
fn::p::nested('str');
fn::p::nested2('obj');
fn::p::nested2('str');
THROW {refusal:'study_decision_generation_changed', class:'stale_premise', n:1};
THROW 'pse:study_decision_generation_changed';
BEGIN; CREATE side:1; LET $r = fn::p::nested('obj'); CREATE side:2; COMMIT;
BEGIN; CREATE side:3; fn::p::nested('str'); CREATE side:4; COMMIT;
BEGIN; CREATE side:5; THROW {refusal:'study_decision_generation_changed', class:'stale_premise', n:1}; CREATE side:6; COMMIT;
BEGIN; CREATE side:7; IF true { THROW 'pse:study_decision_generation_changed' }; COMMIT;
SELECT * FROM side;
-- inside a write statement
CREATE side:9 SET v = fn::p::top('obj');
UPDATE side:9 SET v = fn::p::top('str');
-- object with nested types / special chars
THROW {refusal:"it's \"q\"", n:1.5, d:1dec, b:true, arr:[1,'a'], nested:{z:NONE, y:NULL}, id: side:1, t: d'2026-01-01T00:00:00Z'};
THROW "multi\nline 'quoted'";
