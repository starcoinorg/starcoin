use starcoin_types::state_set as VM1;
use starcoin_vm2_types::state_set as VM2;

pub fn vm1_to_vm2(obj: VM1::StateSet) -> VM2::StateSet {
    VM2::StateSet::new(
        obj.into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}
pub fn vm2_to_vm1(obj: VM2::StateSet) -> VM1::StateSet {
    VM1::StateSet::new(
        obj.into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}
