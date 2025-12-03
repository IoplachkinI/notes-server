use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::service::NoteService;

// Include the generated proto code
pub mod notes {
    tonic::include_proto!("notes");
}

use notes::{
    CreateNoteRequest, DeleteNoteRequest, DeleteNoteResponse, GetAllNotesRequest,
    GetAllNotesResponse, GetNoteRequest, NoteResponse, UpdateNoteRequest,
    note_service_server::{NoteService as NoteServiceTrait, NoteServiceServer},
};

// gRPC service implementation
pub struct GrpcNoteService {
    service: Arc<NoteService>,
}

impl GrpcNoteService {
    pub const fn new(service: Arc<NoteService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl NoteServiceTrait for GrpcNoteService {
    async fn create_note(
        &self,
        request: Request<CreateNoteRequest>,
    ) -> Result<Response<NoteResponse>, Status> {
        let req = request.into_inner();
        let dto_req = crate::dto::CreateNoteRequest {
            content: req.content,
        };

        match self.service.create_note(dto_req).await {
            Ok(note) => Ok(Response::new(NoteResponse {
                id: note.id,
                content: note.content,
            })),
            Err(e) => {
                tracing::error!("Failed to create note: {e}");
                Err(Status::internal("Failed to create note"))
            }
        }
    }

    async fn get_note(
        &self,
        request: Request<GetNoteRequest>,
    ) -> Result<Response<NoteResponse>, Status> {
        let req = request.into_inner();

        match self.service.get_one_note(req.id).await {
            Ok(Some(note)) => Ok(Response::new(NoteResponse {
                id: note.id,
                content: note.content,
            })),
            Ok(None) => Err(Status::not_found("Note not found")),
            Err(e) => {
                tracing::error!("Failed to get note: {e}");
                Err(Status::internal("Failed to get note"))
            }
        }
    }

    async fn get_all_notes(
        &self,
        _request: Request<GetAllNotesRequest>,
    ) -> Result<Response<GetAllNotesResponse>, Status> {
        match self.service.get_all_notes().await {
            Ok(notes) => {
                let grpc_notes: Vec<NoteResponse> = notes
                    .into_iter()
                    .map(|note| NoteResponse {
                        id: note.id,
                        content: note.content,
                    })
                    .collect();

                Ok(Response::new(GetAllNotesResponse { notes: grpc_notes }))
            }
            Err(e) => {
                tracing::error!("Failed to get all notes: {e}");
                Err(Status::internal("Failed to get all notes"))
            }
        }
    }

    async fn update_note(
        &self,
        request: Request<UpdateNoteRequest>,
    ) -> Result<Response<NoteResponse>, Status> {
        let req = request.into_inner();
        let dto_req = crate::dto::UpdateNoteRequest {
            content: req.content,
        };

        match self.service.update_note(req.id, dto_req).await {
            Ok(Some(note)) => Ok(Response::new(NoteResponse {
                id: note.id,
                content: note.content,
            })),
            Ok(None) => Err(Status::not_found("Note not found")),
            Err(e) => {
                tracing::error!("Failed to update note: {e}");
                Err(Status::internal("Failed to update note"))
            }
        }
    }

    async fn delete_note(
        &self,
        request: Request<DeleteNoteRequest>,
    ) -> Result<Response<DeleteNoteResponse>, Status> {
        let req = request.into_inner();

        match self.service.delete_note(req.id).await {
            Ok(true) => Ok(Response::new(DeleteNoteResponse { success: true })),
            Ok(false) => Err(Status::not_found("Note not found")),
            Err(e) => {
                tracing::error!("Failed to delete note: {e}");
                Err(Status::internal("Failed to delete note"))
            }
        }
    }
}

pub fn create_grpc_server(service: Arc<NoteService>) -> NoteServiceServer<GrpcNoteService> {
    NoteServiceServer::new(GrpcNoteService::new(service))
}
