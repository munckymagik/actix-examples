use actix::prelude::Addr;
use actix_web::middleware::session::RequestSession;
use actix_web::{
    http, AsyncResponder, Form, FutureResponse, HttpRequest, HttpResponse, Path,
};
use futures::{future, Future};
use tera::{Context, Tera};

use db::{AllTasks, CreateTask, DbExecutor, DeleteTask, ToggleTask};
use session::{self, FlashMessage};

pub struct AppState {
    pub template: Tera,
    pub db: Addr<DbExecutor>,
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::Found()
        .header(http::header::LOCATION, location)
        .finish()
}

fn bad_request() -> HttpResponse {
    HttpResponse::BadRequest().body("400 Bad Request")
}

pub fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("404 Not Found")
}

fn internal_server_error() -> HttpResponse {
    HttpResponse::InternalServerError().body("500 Internal Server Error")
}

pub fn index(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(AllTasks)
        .from_err()
        .and_then(move |res| match res {
            Ok(tasks) => {
                let mut context = Context::new();
                context.add("tasks", &tasks);

                if let Some(flash) = session::get_flash(&req)? {
                    context.add("msg", &(flash.kind, flash.message));
                    req.session().remove("flash");
                }

                let rendered = req.state()
                    .template
                    .render("index.html.tera", &context)
                    .expect("failed to render template");

                Ok(HttpResponse::Ok().body(rendered))
            }
            Err(_) => Ok(internal_server_error()),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct CreateForm {
    description: String,
}

pub fn create(
    (req, params): (HttpRequest<AppState>, Form<CreateForm>),
) -> FutureResponse<HttpResponse> {
    if params.description.is_empty() {
        future::lazy(move || {
            session::set_flash(
                &req,
                FlashMessage::error("Description cannot be empty"),
            )?;
            Ok(redirect_to("/"))
        }).responder()
    } else {
        req.state()
            .db
            .send(CreateTask {
                description: params.description.clone(),
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    session::set_flash(
                        &req,
                        FlashMessage::success("Task successfully added"),
                    )?;
                    Ok(redirect_to("/"))
                }
                Err(_) => Ok(internal_server_error()),
            })
            .responder()
    }
}

#[derive(Deserialize)]
pub struct UpdateParams {
    id: i32,
}

#[derive(Deserialize)]
pub struct UpdateForm {
    _method: String,
}

pub fn update(
    (req, params, form): (HttpRequest<AppState>, Path<UpdateParams>, Form<UpdateForm>),
) -> FutureResponse<HttpResponse> {
    match form._method.as_ref() {
        "put" => put(req, params),
        "delete" => delete(req, params),
        _ => future::ok(bad_request()).responder(),
    }
}

fn put(
    req: HttpRequest<AppState>,
    params: Path<UpdateParams>,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(ToggleTask { id: params.id })
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => Ok(redirect_to("/")),
            Err(_) => Ok(internal_server_error()),
        })
        .responder()
}

fn delete(
    req: HttpRequest<AppState>,
    params: Path<UpdateParams>,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(DeleteTask { id: params.id })
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => {
                session::set_flash(&req, FlashMessage::success("Task was deleted."))?;
                Ok(redirect_to("/"))
            }
            Err(_) => Ok(internal_server_error()),
        })
        .responder()
}
