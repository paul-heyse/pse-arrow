// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit history obligations, permanent attribution and bounded detail release.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_history_v1::owner_key($kind:string,$owner:string) -> string {
    IF $kind NOT IN ['run','study'] OR $owner='' { THROW 'history owner must identify a run or study'; };
    RETURN $kind+':'+$owner;
};

// Registration covers absence too. Release writes the same guard before fencing.
DEFINE FUNCTION fn::pse_history_v1::check_owner($rpc_expiry:int,$kind:string,$owner:string) -> bool {
    LET $key=fn::pse_history_v1::owner_key($kind,$owner);
    fn::pse_execution_v1::register($rpc_expiry,'history:'+$key);
    IF (SELECT * FROM ONLY type::record('canonical_history_retirements',$key) FOR UPDATE)!=NONE { THROW 'history explicitly retired; detailed response unavailable'; };
    RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::run($rpc_expiry:int,$run:string) -> bool {
    fn::pse_history_v1::check_owner($rpc_expiry,'run',$run);
    LET $parents=SELECT study FROM canonical_history_locators WHERE run=$run LIMIT 3;
    IF array::len($parents)>2 { THROW 'history run association bound exceeded'; };
    FOR $parent IN $parents { fn::pse_history_v1::check_owner($rpc_expiry,'study',$parent.study); };
    RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::point($rpc_expiry:int,$point:string) -> option<object> {
    LET $locator=SELECT * FROM ONLY type::record('canonical_history_locators','point:'+$point);
    IF $locator!=NONE {
        IF $locator.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'history locator issuing authority differs'; };
        fn::pse_history_v1::check_owner($rpc_expiry,'study',$locator.study);
        fn::pse_history_v1::run($rpc_expiry,$locator.run);
    };
    RETURN $locator;
};

// Only membership admission calls this, in the same transaction as membership.
DEFINE FUNCTION fn::pse_history_v1::associate($rpc_expiry:int,$study:string,$run:string,$point:option<string>) -> bool {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_history_v1::run($rpc_expiry,$run);
    fn::pse_execution_v1::touch($rpc_expiry,'history:run:'+$run);
    fn::pse_execution_v1::touch($rpc_expiry,'history:study:'+$study);
    LET $key=IF $point=NONE {'summary:'+$study} ELSE {'point:'+$point};
    LET $old=SELECT * FROM ONLY type::record('canonical_history_locators',$key) FOR UPDATE;
    LET $issuing=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;
    IF $old!=NONE {
        IF $old.study!=$study OR $old.run!=$run OR $old.point!=$point OR $old.issuing_store!=$issuing { THROW 'history membership locator reused'; };
    } ELSE {
        CREATE ONLY type::record('canonical_history_locators',$key) SET key=$key,study=$study,run=$run,point=$point,issuing_store=$issuing;
    };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};

// The stored owner dominates request comparison, including a caller supplying a new run.
DEFINE FUNCTION fn::pse_history_v1::operation($rpc_expiry:int,$key:string,$expected_run:option<string>) -> option<object> {
    LET $old=SELECT * FROM ONLY type::record('canonical_execution_operations',$key) FOR UPDATE;
    IF $old!=NONE {
        fn::pse_history_v1::run($rpc_expiry,$old.run);
        IF (SELECT * FROM ONLY type::record('canonical_result_retirements',$old.run) FOR UPDATE)!=NONE { THROW 'execution results explicitly retired'; };
        IF $old.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'operation issuing authority differs'; };
        IF $old.minimal { THROW 'operation history unavailable; minimal settled attribution only'; };
        IF $expected_run!=NONE AND $old.run!=$expected_run { THROW 'execution operation owner reused'; };
    };
    IF $expected_run!=NONE { fn::pse_history_v1::run($rpc_expiry,$expected_run); IF (SELECT * FROM ONLY type::record('canonical_result_retirements',$expected_run) FOR UPDATE)!=NONE { THROW 'execution results explicitly retired'; }; };
    RETURN $old;
};

DEFINE FUNCTION fn::pse_history_v1::record_operation($rpc_expiry:int,$key:string,$run:string,$attempt:option<string>,$kind:string,$request:bytes,$result:bytes,$study:option<string>,$point:option<string>) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$run);
    LET $parents=SELECT study,point FROM canonical_history_locators WHERE run=$run LIMIT 3;
    IF array::len($parents)>2 { THROW 'operation association bound exceeded'; };
    LET $finalization=IF $attempt=NONE {NONE} ELSE {(SELECT finalization_study FROM ONLY type::record('canonical_attempts',$attempt)).finalization_study};
    LET $selected_study=$study ?? $finalization;
    LET $applicable=IF $point!=NONE {$parents.filter(|$parent|$parent.study=$selected_study AND $parent.point=$point)} ELSE IF $selected_study!=NONE {$parents.filter(|$parent|$parent.study=$selected_study AND $parent.point=NONE)} ELSE {
        LET $points=$parents.filter(|$parent|$parent.point!=NONE);
        IF array::len($points)!=0 {$points} ELSE {$parents}
    };
    IF array::len($applicable)>1 OR (($study!=NONE OR $point!=NONE OR $finalization!=NONE) AND array::len($applicable)!=1) { THROW 'operation exact study/point attribution differs'; };
    LET $parent=IF array::len($applicable)=0 {NONE} ELSE {$applicable[0]};
    CREATE ONLY type::record('canonical_execution_operations',$key) SET key=$key,run=$run,attempt=$attempt,kind=$kind,request=$request,result=$result,
        issuing_store=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store,
        study=$parent.study,point=$parent.point,minimal=false,disposition='settled';
    IF $attempt!=NONE { fn::pse_history_v1::identity($rpc_expiry,'attempt',$attempt,$run); };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::identity($rpc_expiry:int,$category:string,$identity:string,$run:string) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$run);
    LET $key=$category+':'+$identity;
    LET $old=SELECT * FROM ONLY type::record('canonical_history_identities',$key) FOR UPDATE;
    LET $issuing=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;
    IF $old!=NONE {
        IF $old.run!=$run OR $old.issuing_store!=$issuing { THROW 'history identity owner reused'; };
    } ELSE {
        CREATE ONLY type::record('canonical_history_identities',$key) SET key=$key,category=$category,identity=$identity,run=$run,issuing_store=$issuing;
    };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::check_identity($rpc_expiry:int,$category:string,$identity:string) -> bool {
    LET $old=SELECT * FROM ONLY type::record('canonical_history_identities',$category+':'+$identity);
    IF $old!=NONE {
        IF $old.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'history identity issuing authority differs'; };
        fn::pse_history_v1::run($rpc_expiry,$old.run);
    };
    RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::effect_free($rpc_expiry:int,$point:object) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$point.run);
    fn::pse_execution_v1::touch($rpc_expiry,'execution-run:'+$point.run);
    LET $run=SELECT * FROM ONLY type::record('canonical_runs',$point.run) FOR UPDATE;
    IF !$point.settled OR $point.assigned OR $point.attempt!=NONE OR $run=NONE OR $run.current_attempt!=NONE OR $run.current_generation!=0dec { THROW 'effect-free settlement requires original unclaimed occurrence'; };
    fn::pse_history_v1::identity($rpc_expiry,'effect-free-run',$point.run,$point.run);
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::read_run($rpc_expiry:int,$key:string) -> option<object> {
    fn::pse_history_v1::run($rpc_expiry,$key);
    RETURN SELECT * FROM ONLY type::record('canonical_runs',$key);
};

DEFINE FUNCTION fn::pse_history_v1::read_attempt($rpc_expiry:int,$key:string) -> option<object> {
    fn::pse_history_v1::check_identity($rpc_expiry,'attempt',$key);
    LET $row=SELECT * FROM ONLY type::record('canonical_attempts',$key);
    IF $row!=NONE { fn::pse_history_v1::run($rpc_expiry,$row.run); };
    RETURN $row;
};

DEFINE FUNCTION fn::pse_history_v1::read_manifest($rpc_expiry:int,$key:string) -> option<object> {
    fn::pse_history_v1::check_identity($rpc_expiry,'manifest',$key);
    RETURN SELECT * FROM ONLY type::record('canonical_result_manifests',$key);
};

DEFINE FUNCTION fn::pse_history_v1::retain($rpc_expiry:int,$key:string,$kind:string,$owner:string,$purpose:string,$attribution:string) -> bool {
    IF $key='' OR $purpose NOT IN ['audit','unsettled'] OR $attribution='' { THROW 'history obligation requires exact owner and attribution'; };
    LET $scope=fn::pse_history_v1::owner_key($kind,$owner);
    fn::pse_history_v1::check_owner($rpc_expiry,$kind,$owner);
    IF $kind='run' {
        fn::pse_history_v1::run($rpc_expiry,$owner);
        LET $parents=SELECT study FROM canonical_history_locators WHERE run=$owner LIMIT 3;
        IF array::len($parents)>2 { THROW 'history obligation association bound'; };
        FOR $parent IN $parents { fn::pse_execution_v1::touch($rpc_expiry,'history:study:'+$parent.study); };
    };
    fn::pse_execution_v1::touch($rpc_expiry,'history:'+$scope);
    LET $header=IF $kind='run' {SELECT key FROM ONLY type::record('canonical_runs',$owner)} ELSE {SELECT key FROM ONLY type::record('canonical_studies',$owner)};
    IF $header=NONE { THROW 'history obligation owner unavailable'; };
    IF $kind='study' {
        LET $retired=SELECT key FROM canonical_history_locators WHERE study=$owner AND type::record('canonical_history_retirements','run:'+run)!=NONE LIMIT 1;
        IF array::len($retired)!=0 { THROW 'study audit cannot restore retired member history'; };
    };
    LET $old=SELECT * FROM ONLY type::record('canonical_history_obligations',$key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.owner_kind!=$kind OR $old.owner!=$owner OR $old.purpose!=$purpose OR $old.attribution!=$attribution { THROW 'history obligation identity reused'; };
        IF $old.released { THROW 'history obligation explicitly released'; };
        fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
    };
    CREATE ONLY type::record('canonical_history_obligations',$key) SET key=$key,owner_kind=$kind,owner=$owner,purpose=$purpose,attribution=$attribution,released=false,issuing_store=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;
    LET $count=SELECT * FROM ONLY type::record('canonical_history_obligation_counts',$scope) FOR UPDATE;
    UPSERT ONLY type::record('canonical_history_obligation_counts',$scope) SET key=$scope,count=($count.count ?? 0dec)+1dec;
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::drop_obligation($rpc_expiry:int,$key:string,$kind:string,$owner:string,$purpose:string,$attribution:string) -> bool {
    LET $scope=fn::pse_history_v1::owner_key($kind,$owner);
    fn::pse_execution_v1::touch($rpc_expiry,'history:'+$scope);
    LET $old=SELECT * FROM ONLY type::record('canonical_history_obligations',$key) FOR UPDATE;
    IF $old=NONE {
        CREATE ONLY type::record('canonical_history_obligations',$key) SET key=$key,owner_kind=$kind,owner=$owner,purpose=$purpose,attribution=$attribution,released=true,issuing_store=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;
        fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
    };
    IF $old.owner_kind!=$kind OR $old.owner!=$owner OR $old.purpose!=$purpose OR $old.attribution!=$attribution OR $old.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'history obligation release differs'; };
    IF $old.released { fn::pse_execution_v1::deadline($rpc_expiry); RETURN true; };
    LET $count=SELECT * FROM ONLY type::record('canonical_history_obligation_counts',$scope) FOR UPDATE;
    IF $count=NONE OR $count.count<1dec { THROW 'history obligation count unavailable'; };
    UPDATE ONLY type::record('canonical_history_obligations',$key) SET released=true;
    UPDATE ONLY type::record('canonical_history_obligation_counts',$scope) SET count-=1dec;
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};

DEFINE FUNCTION fn::pse_history_v1::release($rpc_expiry:int,$kind:string,$owner:string) -> object {
    LET $key=fn::pse_history_v1::owner_key($kind,$owner);
    fn::pse_execution_v1::touch($rpc_expiry,'history:'+$key);
    LET $old=SELECT * FROM ONLY type::record('canonical_history_retirements',$key) FOR UPDATE;
    IF $old!=NONE { fn::pse_execution_v1::deadline($rpc_expiry); RETURN $old; };
    LET $count=SELECT * FROM ONLY type::record('canonical_history_obligation_counts',$key) FOR UPDATE;
    IF ($count.count ?? 0dec)!=0dec { THROW 'selected audit or unsettled history obligation remains'; };
    LET $header=IF $kind='run' {SELECT * FROM ONLY type::record('canonical_runs',$owner)} ELSE {SELECT * FROM ONLY type::record('canonical_studies',$owner)};
    IF $header=NONE { THROW 'history release requires its original owner'; };
    IF $kind='run' {
        fn::pse_execution_v1::touch($rpc_expiry,'execution-run:'+$owner);
        LET $retired=SELECT * FROM ONLY type::record('canonical_result_retirements',$owner) FOR UPDATE;
        IF $retired=NONE OR !$retired.complete OR ($header.terminal_attempt=NONE AND (SELECT * FROM ONLY type::record('canonical_history_identities','effect-free-run:'+$owner))=NONE) { THROW 'history release requires settled retired run payloads'; };
        LET $parents=SELECT study FROM canonical_history_locators WHERE run=$owner LIMIT 3;
        IF array::len($parents)>2 { THROW 'history release association bound'; };
        FOR $parent IN $parents {
            fn::pse_execution_v1::register($rpc_expiry,'history:study:'+$parent.study);
            LET $parent_count=SELECT * FROM ONLY type::record('canonical_history_obligation_counts','study:'+$parent.study) FOR UPDATE;
            IF ($parent_count.count ?? 0dec)!=0dec { THROW 'study audit retains run history'; };
        };
    } ELSE {
        fn::pse_execution_v1::touch($rpc_expiry,'study:'+$owner);
        IF !$header.terminal OR (SELECT * FROM ONLY type::record('canonical_study_retirements',$owner) FOR UPDATE)=NONE { THROW 'history release requires terminal retired study'; };
        LET $protected=SELECT key FROM canonical_history_locators WHERE study=$owner AND (type::record('canonical_history_obligation_counts','run:'+run).count ?? 0dec)>0dec LIMIT 1;
        IF array::len($protected)!=0 { THROW 'point or summary audit retains study history'; };
        LET $unsettled=SELECT key FROM canonical_history_locators WHERE study=$owner AND (type::record('canonical_result_retirements',run).complete ?? false)=false LIMIT 1;
        IF array::len($unsettled)!=0 { THROW 'study history requires all linked run payloads settled'; };
    };
    LET $outcome=IF $kind='run' {$header.terminal_class ?? 'not-executed'} ELSE {IF $header.cancelled {'cancelled'} ELSE {'terminal'}};
    LET $result=CREATE ONLY type::record('canonical_history_retirements',$key) SET key=$key,owner_kind=$kind,owner=$owner,issuing_store=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store,outcome=$outcome,complete=false,after_locator=NONE;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $result;
};

DEFINE FUNCTION fn::pse_history_v1::collect($rpc_expiry:int,$kind:string,$owner:string) -> object {
    LET $key=fn::pse_history_v1::owner_key($kind,$owner);
    fn::pse_execution_v1::touch($rpc_expiry,'history:'+$key);
    LET $fence=SELECT * FROM ONLY type::record('canonical_history_retirements',$key) FOR UPDATE;
    IF $fence=NONE { THROW 'explicit history release required'; };
    IF $fence.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'history collector issuing authority differs'; };
    IF $fence.complete { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:true,removed:0dec,compacted:0dec}; };
    IF $kind='run' {
        LET $operations=SELECT key FROM canonical_execution_operations WHERE run=$owner AND minimal=false ORDER BY key LIMIT 64;
        IF array::len($operations)!=0 {
            FOR $operation IN $operations { UPDATE ONLY type::record('canonical_execution_operations',$operation.key) SET request=type::bytes(''),result=type::bytes(''),minimal=true; };
            fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:0dec,compacted:<decimal>array::len($operations)};
        };
        LET $attempts=SELECT id FROM canonical_attempts WHERE run=$owner ORDER BY generation LIMIT 64;
        IF array::len($attempts)!=0 { DELETE $attempts.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:<decimal>array::len($attempts),compacted:0dec}; };
        LET $sources=SELECT id FROM canonical_run_sources WHERE run=$owner LIMIT 64;
        IF array::len($sources)!=0 { DELETE $sources.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:<decimal>array::len($sources),compacted:0dec}; };
        LET $edges=SELECT id FROM canonical_problem_runs WHERE run=$owner LIMIT 64;
        IF array::len($edges)!=0 { DELETE $edges.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:<decimal>array::len($edges),compacted:0dec}; };
        DELETE ONLY type::record('canonical_runs',$owner);
    } ELSE {
        LET $children=SELECT key,run FROM canonical_history_locators WHERE study=$owner AND key>($fence.after_locator ?? '') ORDER BY key LIMIT 1;
        IF array::len($children)!=0 {
            LET $child=$children[0];
            fn::pse_history_v1::release($rpc_expiry,'run',$child.run);
            LET $page=fn::pse_history_v1::collect($rpc_expiry,'run',$child.run);
            IF $page.complete { UPDATE ONLY type::record('canonical_history_retirements',$key) SET after_locator=$child.key; };
            fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:$page.removed,compacted:$page.compacted};
        };
        LET $inputs=SELECT id FROM canonical_study_inputs WHERE study=$owner ORDER BY key LIMIT 64;
        IF array::len($inputs)!=0 { DELETE $inputs.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:<decimal>array::len($inputs),compacted:0dec}; };
        LET $dependencies=SELECT id FROM canonical_study_dependencies WHERE study=$owner ORDER BY key LIMIT 64;
        IF array::len($dependencies)!=0 { DELETE $dependencies.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:<decimal>array::len($dependencies),compacted:0dec}; };
        LET $readiness=SELECT id FROM canonical_study_readiness WHERE study=$owner ORDER BY ordinal LIMIT 64;
        IF array::len($readiness)!=0 { DELETE $readiness.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:<decimal>array::len($readiness),compacted:0dec}; };
        LET $points=SELECT id FROM canonical_study_points WHERE study=$owner ORDER BY ordinal LIMIT 64;
        IF array::len($points)!=0 { DELETE $points.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,removed:<decimal>array::len($points),compacted:0dec}; };
        DELETE ONLY type::record('canonical_study_readiness_cursors',$owner);
        DELETE ONLY type::record('canonical_studies',$owner);
    };
    UPDATE ONLY type::record('canonical_history_retirements',$key) SET complete=true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:true,removed:1dec,compacted:0dec};
};
"#;
