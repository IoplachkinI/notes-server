use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{dto, service::NoteService};

// Request envelope

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(
    rename = "Envelope",
    namespaces = {
      "soap" = "http://www.w3.org/2003/05/soap-envelope",
    },
    prefix = "soap"
  )]
pub struct SoapEnvelope {
    #[yaserde(rename = "Body", prefix = "soap")]
    pub body: SoapBody,
}

// Request body

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct SoapBody {
    /// ``CreateNote`` operation request
    #[yaserde(rename = "CreateNote", prefix = "m")]
    pub create: Option<CreateNoteRequest>,

    /// ``GetOneNote`` operation request
    #[yaserde(rename = "GetNote", prefix = "m")]
    pub get_one: Option<GetOneNoteRequest>,

    /// ``GetAllNotes`` operation request
    #[yaserde(rename = "UpdateNote", prefix = "m")]
    pub get_all: Option<GetAllNotesRequest>,

    /// ``UpdateNote`` operation request
    #[yaserde(rename = "UpdateNote", prefix = "m")]
    pub update: Option<UpdateNoteRequest>,

    /// ``DeleteNote`` operation request
    #[yaserde(rename = "DeleteNote", prefix = "m")]
    pub delete: Option<DeleteNoteRequest>,
}

// Request content variants

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct CreateNoteRequest {
    #[yaserde(rename = "Content", prefix = "m")]
    pub content: String,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct GetOneNoteRequest {
    #[yaserde(rename = "Id", prefix = "m")]
    pub id: i64,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct GetAllNotesRequest;

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct UpdateNoteRequest {
    #[yaserde(rename = "Id", prefix = "m")]
    pub id: i64,

    #[yaserde(rename = "Content", prefix = "m")]
    pub content: String,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct DeleteNoteRequest {
    #[yaserde(rename = "Id", prefix = "m")]
    pub id: i64,
}

// Enum for all operation types
enum NoteOperationRequest {
    Create(CreateNoteRequest),
    GetOne(GetOneNoteRequest),
    GetAll,
    Update(UpdateNoteRequest),
    Delete(DeleteNoteRequest),
}

fn to_operation(body: SoapBody) -> Option<NoteOperationRequest> {
    if let Some(c) = body.create {
        return Some(NoteOperationRequest::Create(c));
    }
    if let Some(g) = body.get_one {
        return Some(NoteOperationRequest::GetOne(g));
    }
    if let Some(_g) = body.get_all {
        return Some(NoteOperationRequest::GetAll);
    }
    if let Some(u) = body.update {
        return Some(NoteOperationRequest::Update(u));
    }
    if let Some(d) = body.delete {
        return Some(NoteOperationRequest::Delete(d));
    }
    None
}

// Common response elements

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct NoteResponse {
    #[yaserde(rename = "Note", prefix = "m")]
    pub note: NoteResponseXml,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct NoteResponseXml {
    #[yaserde(rename = "Id", prefix = "m")]
    pub id: i64,

    #[yaserde(rename = "Content", prefix = "m")]
    pub content: String,
}

// CreateResponse

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(
    rename = "Envelope",
    namespaces = {
      "soap" = "http://www.w3.org/2003/05/soap-envelope",
    },
    prefix = "soap"
  )]
pub struct CreateNoteResponseEnvelope {
    #[yaserde(rename = "Body", prefix = "soap")]
    pub body: CreateNoteResponseBody,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct CreateNoteResponseBody {
    #[yaserde(rename = "CreateNoteResponse", prefix = "m")]
    pub response: NoteResponse,
}

// GetOneResponse

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(
    rename = "Envelope",
    namespaces = {
      "soap" = "http://www.w3.org/2003/05/soap-envelope",
    },
    prefix = "soap"
  )]
pub struct GetOneNoteResponseEnvelope {
    #[yaserde(rename = "Body", prefix = "m")]
    pub body: GetOneNoteResponseBody,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct GetOneNoteResponseBody {
    #[yaserde(rename = "GetOneNoteResponse", prefix = "m")]
    pub response: NoteResponse,
}

// GetAllResponse

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(
    rename = "Envelope",
    namespaces = {
      "soap" = "http://www.w3.org/2003/05/soap-envelope",
    },
    prefix = "soap"
  )]
pub struct GetAllNotesResponseEnvelope {
    #[yaserde(rename = "Body", prefix = "soap")]
    pub body: GetAllNotesResponseBody,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct GetAllNotesResponseBody {
    #[yaserde(rename = "GetAllNotesResponse", prefix = "m")]
    pub response: GetAllNotesResponse,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct GetAllNotesResponse {
    #[yaserde(rename = "Note", prefix = "m")]
    pub notes: Vec<NoteResponseXml>,
}

// UpdateResponse

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(
    rename = "Envelope",
    namespaces = {
      "soap" = "http://www.w3.org/2003/05/soap-envelope",
    },
    prefix = "soap"
  )]
pub struct UpdateNoteResponseEnvelope {
    #[yaserde(rename = "Body", prefix = "soap")]
    pub body: UpdateNoteResponseBody,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct UpdateNoteResponseBody {
    #[yaserde(rename = "UpdateNoteResponse", prefix = "m")]
    pub response: NoteResponse,
}

// DeleteResponse

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(
    rename = "Envelope",
    namespaces = {
      "soap" = "http://www.w3.org/2003/05/soap-envelope",
    },
    prefix = "soap"
  )]
pub struct DeleteNoteResponseEnvelope {
    #[yaserde(rename = "Body", prefix = "soap")]
    pub body: DeleteNoteResponseBody,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct DeleteNoteResponseBody {
    #[yaserde(rename = "DeleteNoteResponse", prefix = "m")]
    pub response: DeleteNoteResponse,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(namespaces = {"m" = "https://notes-server/soap/v1"})]
pub struct DeleteNoteResponse {}

/// Main SOAP handler entrypoint
pub async fn handle_request(State(service): State<Arc<NoteService>>, body: Bytes) -> Response {
    let Ok(body_str) = std::str::from_utf8(&body) else {
        return (StatusCode::BAD_REQUEST, "Request body must be valid UTF-8").into_response();
    };

    let envelope: SoapEnvelope = match yaserde::de::from_str(body_str) {
        Ok(env) => env,
        Err(e) => {
            tracing::error!("Failed to deserialize SOAP envelope: {e}");
            let fault_xml = build_soap_fault(
                SoapFaultCode::Client,
                "Invalid SOAP XML envelope: request body could not be parsed",
            );
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/xml; charset=utf-8")],
                fault_xml,
            )
                .into_response();
        }
    };

    match to_operation(envelope.body) {
        Some(NoteOperationRequest::Create(c)) => handle_create_note(&service, c).await,
        Some(NoteOperationRequest::GetOne(g)) => handle_get_one_note(&service, g).await,
        Some(NoteOperationRequest::GetAll) => handle_get_all_notes(&service).await,
        Some(NoteOperationRequest::Update(u)) => handle_update_note(&service, u).await,
        Some(NoteOperationRequest::Delete(d)) => handle_delete_note(&service, d).await,
        None => {
            let fault_xml = build_soap_fault(SoapFaultCode::Client, "Unsupported operation");
            (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/xml; charset=utf-8")],
                fault_xml,
            )
                .into_response()
        }
    }
}

/// Common SOAP 1.1 fault codes.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum SoapFaultCode {
    /// The message was incorrectly formed or contained incorrect information.
    Client,
    /// The message could not be processed for reasons not directly attributable to the client.
    Server,
    /// An immediate child element of the Header was not understood.
    MustUnderstand,
    /// The SOAP Envelope namespace is not supported.
    VersionMismatch,
}

impl SoapFaultCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Client => "Client",
            Self::Server => "Server",
            Self::MustUnderstand => "MustUnderstand",
            Self::VersionMismatch => "VersionMismatch",
        }
    }
}

fn handle_serialization_error(e: &String) -> Response {
    tracing::error!("Failed to serialize SOAP response: {e}");
    let fault_xml = build_soap_fault(SoapFaultCode::Server, "Failed to serialize SOAP response");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [("Content-Type", "text/xml; charset=utf-8")],
        fault_xml,
    )
        .into_response()
}

fn handle_internal_error(err: &tokio_postgres::Error, custom_error_string: &str) -> Response {
    tracing::error!("{custom_error_string}: {err}");
    let fault_xml = build_soap_fault(SoapFaultCode::Server, custom_error_string);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [("Content-Type", "text/xml; charset=utf-8")],
        fault_xml,
    )
        .into_response()
}

fn handle_not_found_error() -> Response {
    tracing::error!("Note not found");
    let fault_xml = build_soap_fault(SoapFaultCode::Server, "Note not found");
    (
        StatusCode::NOT_FOUND,
        [("Content-Type", "text/xml; charset=utf-8")],
        fault_xml,
    )
        .into_response()
}

fn build_ok_response(xml_body: String) -> Response {
    (
        StatusCode::OK,
        [("Content-Type", "text/xml; charset=utf-8")],
        xml_body,
    )
        .into_response()
}

fn build_soap_fault(fault_code: SoapFaultCode, fault_string: &str) -> String {
    format!(
        r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
  <soap:Body>
    <soap:Fault>
      <faultcode>{fault_code}</faultcode>
      <faultstring>{fault_string}</faultstring>
    </soap:Fault>
  </soap:Body>
</soap:Envelope>"#,
        fault_code = fault_code.as_str()
    )
}

async fn handle_create_note(service: &NoteService, req: CreateNoteRequest) -> Response {
    let dto_req = dto::CreateNoteRequest {
        content: req.content,
    };

    match service.create_note(dto_req).await {
        Ok(note) => {
            let note_xml = NoteResponseXml {
                id: note.id,
                content: note.content,
            };

            let response_envelope = CreateNoteResponseEnvelope {
                body: CreateNoteResponseBody {
                    response: NoteResponse { note: note_xml },
                },
            };

            let xml_body = match yaserde::ser::to_string(&response_envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&e),
            };

            build_ok_response(xml_body)
        }
        Err(e) => handle_internal_error(&e, "Failed to create note"),
    }
}

async fn handle_get_one_note(service: &NoteService, req: GetOneNoteRequest) -> Response {
    match service.get_one_note(req.id).await {
        Ok(Some(note)) => {
            let note_xml = NoteResponseXml {
                id: note.id,
                content: note.content,
            };

            let response_envelope = GetOneNoteResponseEnvelope {
                body: GetOneNoteResponseBody {
                    response: NoteResponse { note: note_xml },
                },
            };

            let xml_body = match yaserde::ser::to_string(&response_envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&e),
            };

            build_ok_response(xml_body)
        }
        Ok(None) => handle_not_found_error(),
        Err(e) => handle_internal_error(&e, "Failed to get note"),
    }
}

async fn handle_get_all_notes(service: &NoteService) -> Response {
    match service.get_all_notes().await {
        Ok(notes) => {
            let mut notes_resp: Vec<NoteResponseXml> = Vec::new();

            for note in notes {
                notes_resp.push(NoteResponseXml {
                    id: note.id,
                    content: note.content,
                });
            }

            let response_envelope = GetAllNotesResponseEnvelope {
                body: GetAllNotesResponseBody {
                    response: GetAllNotesResponse { notes: notes_resp },
                },
            };

            let xml_body = match yaserde::ser::to_string(&response_envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&e),
            };

            build_ok_response(xml_body)
        }
        Err(e) => handle_internal_error(&e, "Failed to get note"),
    }
}

async fn handle_update_note(service: &NoteService, req: UpdateNoteRequest) -> Response {
    let dto_req = dto::UpdateNoteRequest {
        content: req.content,
    };

    match service.update_note(req.id, dto_req).await {
        Ok(Some(note)) => {
            let note_xml = NoteResponseXml {
                id: note.id,
                content: note.content,
            };

            let response_envelope = UpdateNoteResponseEnvelope {
                body: UpdateNoteResponseBody {
                    response: NoteResponse { note: note_xml },
                },
            };

            let xml_body = match yaserde::ser::to_string(&response_envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&e),
            };

            build_ok_response(xml_body)
        }
        Ok(None) => handle_not_found_error(),
        Err(e) => handle_internal_error(&e, "Failed to update note"),
    }
}

async fn handle_delete_note(service: &NoteService, req: DeleteNoteRequest) -> Response {
    match service.delete_note(req.id).await {
        Ok(true) => {
            let response_envelope = DeleteNoteResponseEnvelope {
                body: DeleteNoteResponseBody {
                    response: DeleteNoteResponse {},
                },
            };

            let xml_body = match yaserde::ser::to_string(&response_envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&e),
            };

            build_ok_response(xml_body)
        }
        Ok(false) => handle_not_found_error(),
        Err(e) => handle_internal_error(&e, "Failed to delete note"),
    }
}
