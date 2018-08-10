use futures_core::{
    future::FutureObj,
    task::{Executor, SpawnObjError},
};

pub struct NoSpawn;

impl Executor for NoSpawn {
    fn spawn_obj(
        &mut self,
        _future: FutureObj<'static, ()>,
    ) -> Result<(), SpawnObjError> {
        panic!("should not spawn")
    }
}
