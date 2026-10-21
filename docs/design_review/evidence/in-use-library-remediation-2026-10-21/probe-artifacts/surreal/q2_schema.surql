DEFINE FUNCTION fn::p::top($m: string) { IF $m = 'obj' { THROW {refusal:'study_decision_generation_changed', class:'stale_premise', n:1} } ELSE { THROW 'pse:study_decision_generation_changed' } };
DEFINE FUNCTION fn::p::inif($m: string, $c: bool) { IF $c { IF $m = 'obj' { THROW {refusal:'study_decision_generation_changed', class:'stale_premise', n:1} } ELSE { THROW 'pse:study_decision_generation_changed' } }; RETURN 'no' };
DEFINE FUNCTION fn::p::infor($m: string) { FOR $i IN [1,2,3] { IF $i = 2 { IF $m = 'obj' { THROW {refusal:'study_decision_generation_changed', class:'stale_premise', n:1} } ELSE { THROW 'pse:study_decision_generation_changed' } } }; RETURN 'no' };
DEFINE FUNCTION fn::p::nested($m: string) { LET $x = fn::p::infor($m); RETURN $x };
DEFINE FUNCTION fn::p::nested2($m: string) { FOR $j IN [1] { LET $x = fn::p::nested($m) }; RETURN 'no' };
DEFINE TABLE side SCHEMALESS;
