use crate::actor::HandleResult;

#[inline]
pub fn process_iteration_result(actor_name: &String, res: HandleResult) {
    if let Err(err) = res {
        tracing::error!("Error during process iteration: {}", err);
    } else {
        tracing::trace!("{actor_name:?} successful iteration");
    }
}
