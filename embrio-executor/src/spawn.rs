use core::{
    future::FutureObj,
    task::{Spawn, SpawnObjError},
};

pub struct NoSpawn;

impl Spawn for NoSpawn {
    fn spawn_obj(
        &mut self,
        _future: FutureObj<'static, ()>,
    ) -> Result<(), SpawnObjError> {
        panic!("should not spawn")
    }
}
