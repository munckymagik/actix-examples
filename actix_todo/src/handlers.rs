use actix::prelude::{Handler, Message};
use actix_web::{error, Error};

use db::DbExecutor;
use model::{NewTask, Task};

pub struct CreateTask {
    pub description: String,
}

impl Message for CreateTask {
    type Result = Result<(), Error>;
}

impl Handler<CreateTask> for DbExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, todo: CreateTask, _: &mut Self::Context) -> Self::Result {
        let new_task = NewTask {
            description: todo.description,
        };
        Task::insert(new_task, self.get_conn()?)
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Error inserting task"))
    }
}

pub struct AllTasks;

impl Message for AllTasks {
    type Result = Result<Vec<Task>, Error>;
}

impl Handler<AllTasks> for DbExecutor {
    type Result = Result<Vec<Task>, Error>;

    fn handle(&mut self, _: AllTasks, _: &mut Self::Context) -> Self::Result {
        Task::all(self.get_conn()?)
            .map_err(|_| error::ErrorInternalServerError("Error inserting task"))
    }
}

pub struct ToggleTask {
    pub id: i32,
}

impl Message for ToggleTask {
    type Result = Result<(), Error>;
}

impl Handler<ToggleTask> for DbExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, task: ToggleTask, _: &mut Self::Context) -> Self::Result {
        Task::toggle_with_id(task.id, self.get_conn()?)
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Error inserting task"))
    }
}

pub struct DeleteTask {
    pub id: i32,
}

impl Message for DeleteTask {
    type Result = Result<(), Error>;
}

impl Handler<DeleteTask> for DbExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, task: DeleteTask, _: &mut Self::Context) -> Self::Result {
        Task::delete_with_id(task.id, self.get_conn()?)
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Error inserting task"))
    }
}
