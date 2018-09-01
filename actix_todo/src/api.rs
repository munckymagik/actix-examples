use actix::prelude::Addr;
use actix_web::middleware::session::RequestSession;
use actix_web::middleware::Response;
use actix_web::{
    error, http, AsyncResponder, Form, FutureResponse, HttpRequest, HttpResponse, Path,
    Result,
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

pub fn bad_request<S>(_: &HttpRequest<S>, resp: HttpResponse) -> Result<Response> {
    Ok(Response::Done(resp.into_builder().body("400 Bad Request")))
}


pub fn not_found<S>(_: &HttpRequest<S>, resp: HttpResponse) -> Result<Response> {
    Ok(Response::Done(resp.into_builder().body("404 Not Found")))
}

pub fn internal_server_error<S>(
    _: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    Ok(Response::Done(
        resp.into_builder().body("500 Internal Server Error"),
    ))
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
                    .map_err(|e| {
                        error::ErrorInternalServerError(e.description().to_owned())
                    })?;

                Ok(HttpResponse::Ok().body(rendered))
            }
            Err(e) => Err(e),
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
                Err(e) => Err(e),
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

pub fn update_with_reinterpreted_method(
    (req, params, form): (HttpRequest<AppState>, Path<UpdateParams>, Form<UpdateForm>),
) -> FutureResponse<HttpResponse> {
    match form._method.as_ref() {
        "put" => update(req, params),
        "delete" => delete(req, params),
        unsupported_method => {
            let msg = format!("Unsupported HTTP method: {}", unsupported_method);
            future::err(error::ErrorBadRequest(msg)).responder()
        }
    }
}

fn update(
    req: HttpRequest<AppState>,
    params: Path<UpdateParams>,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(ToggleTask { id: params.id })
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => Ok(redirect_to("/")),
            Err(e) => Err(e),
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
            Err(e) => Err(e),
        })
        .responder()
}
