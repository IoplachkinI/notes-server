use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{dto, service::NoteService};

// Request envelope

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "Envelope")]
#[serde(rename_all = "camelCase")]
pub struct SoapEnvelope {
    #[serde(rename = "Body")]
    pub body: SoapBody,
}

// Request body

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoapBody {
    /// ``CreateNote`` operation request
    #[serde(rename = "CreateNote")]
    pub create: Option<CreateNoteRequest>,

    /// ``GetOneNote`` operation request
    #[serde(rename = "GetNote")]
    pub get_one: Option<GetOneNoteRequest>,

    /// ``GetAllNotes`` operation request
    #[serde(rename = "GetAllNotes")]
    pub get_all: Option<GetAllNotesRequest>,

    /// ``UpdateNote`` operation request
    #[serde(rename = "UpdateNote")]
    pub update: Option<UpdateNoteRequest>,

    /// ``DeleteNote`` operation request
    #[serde(rename = "DeleteNote")]
    pub delete: Option<DeleteNoteRequest>,
}

// Request content variants

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteRequest {
    #[serde(rename = "Content")]
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOneNoteRequest {
    #[serde(rename = "Id")]
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetAllNotesRequest;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteRequest {
    #[serde(rename = "Id")]
    pub id: i64,

    #[serde(rename = "Content")]
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNoteRequest {
    #[serde(rename = "Id")]
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

#[derive(Debug, Serialize)]
pub struct NoteResponseXml {
    #[serde(rename = "m:Id")]
    pub id: i64,

    #[serde(rename = "m:Content")]
    pub content: String,
}

// CreateResponse

#[derive(Debug, Serialize)]
#[serde(rename = "m:CreateNoteResponse")]
pub struct CreateNoteResponse {
    #[serde(rename = "@xmlns:m")]
    pub m_ns: String,
    #[serde(rename = "m:Note")]
    pub note: NoteResponseXml,
}

// GetOneResponse

#[derive(Debug, Serialize)]
#[serde(rename = "m:GetOneNoteResponse")]
pub struct GetOneNoteResponse {
    #[serde(rename = "@xmlns:m")]
    pub m_ns: String,
    #[serde(rename = "m:Note")]
    pub note: NoteResponseXml,
}

// GetAllResponse

#[derive(Debug, Serialize)]
#[serde(rename = "m:GetAllNotesResponse")]
pub struct GetAllNotesResponse {
    #[serde(rename = "@xmlns:m")]
    pub m_ns: String,
    #[serde(rename = "m:Note")]
    pub notes: Vec<NoteResponseXml>,
}

// UpdateResponse

#[derive(Debug, Serialize)]
#[serde(rename = "m:UpdateNoteResponse")]
pub struct UpdateNoteResponse {
    #[serde(rename = "@xmlns:m")]
    pub m_ns: String,
    #[serde(rename = "m:Note")]
    pub note: NoteResponseXml,
}

// DeleteResponse

#[derive(Debug, Serialize)]
#[serde(rename = "m:DeleteNoteResponse")]
pub struct DeleteNoteResponse {
    #[serde(rename = "@xmlns:m")]
    pub m_ns: String,
}

/// Main SOAP handler entrypoint
pub async fn handle_request(State(service): State<Arc<NoteService>>, body: Bytes) -> Response {
    let Ok(body_str) = std::str::from_utf8(&body) else {
        return (StatusCode::BAD_REQUEST, "Request body must be valid UTF-8").into_response();
    };

    let envelope: SoapEnvelope = match serde_xml_rs::from_str(body_str) {
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
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" soap:encodingStyle="http://www.w3.org/2003/05/soap-encoding">
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

#[derive(Debug, Serialize)]
#[serde(rename = "soap:Envelope", rename_all = "kebab-case")]
struct CreateNoteEnvelope {
    #[serde(rename = "@xmlns:soap")]
    soap_ns: String,
    #[serde(rename = "@soap:encodingStyle")]
    encoding_style: String,
    #[serde(rename = "soap:Body")]
    body: CreateNoteBody,
}

#[derive(Debug, Serialize)]
struct CreateNoteBody {
    #[serde(rename = "m:CreateNoteResponse")]
    response: CreateNoteResponse,
}

async fn handle_create_note(service: &NoteService, req: CreateNoteRequest) -> Response {
    let dto_req = dto::CreateNoteRequest {
        content: req.content,
    };

    match service.create_note(dto_req).await {
        Ok(note) => {
            let response = CreateNoteResponse {
                m_ns: "https://notes-server/soap/v1".to_string(),
                note: NoteResponseXml {
                    id: note.id,
                    content: note.content,
                },
            };

            let envelope = CreateNoteEnvelope {
                soap_ns: "http://www.w3.org/2003/05/soap-envelope".to_string(),
                encoding_style: "http://www.w3.org/2003/05/soap-encoding".to_string(),
                body: CreateNoteBody { response },
            };

            let xml_body = match quick_xml::se::to_string(&envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&format!("{e}")),
            };

            build_ok_response(xml_body)
        }
        Err(e) => handle_internal_error(&e, "Failed to create note"),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "soap:Envelope")]
struct GetOneNoteEnvelope {
    #[serde(rename = "@xmlns:soap")]
    soap_ns: String,
    #[serde(rename = "@soap:encodingStyle")]
    encoding_style: String,
    #[serde(rename = "soap:Body")]
    body: GetOneNoteBody,
}

#[derive(Debug, Serialize)]
struct GetOneNoteBody {
    #[serde(rename = "m:GetOneNoteResponse")]
    response: GetOneNoteResponse,
}

async fn handle_get_one_note(service: &NoteService, req: GetOneNoteRequest) -> Response {
    match service.get_one_note(req.id).await {
        Ok(Some(note)) => {
            let response = GetOneNoteResponse {
                m_ns: "https://notes-server/soap/v1".to_string(),
                note: NoteResponseXml {
                    id: note.id,
                    content: note.content,
                },
            };

            let envelope = GetOneNoteEnvelope {
                soap_ns: "http://www.w3.org/2003/05/soap-envelope".to_string(),
                encoding_style: "http://www.w3.org/2003/05/soap-encoding".to_string(),
                body: GetOneNoteBody { response },
            };

            let xml_body = match quick_xml::se::to_string(&envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&format!("{e}")),
            };

            build_ok_response(xml_body)
        }
        Ok(None) => handle_not_found_error(),
        Err(e) => handle_internal_error(&e, "Failed to get note"),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "soap:Envelope")]
struct GetAllNotesEnvelope {
    #[serde(rename = "@xmlns:soap")]
    soap_ns: String,
    #[serde(rename = "@soap:encodingStyle")]
    encoding_style: String,
    #[serde(rename = "soap:Body")]
    body: GetAllNotesBody,
}

#[derive(Debug, Serialize)]
struct GetAllNotesBody {
    #[serde(rename = "m:GetAllNotesResponse")]
    response: GetAllNotesResponse,
}

async fn handle_get_all_notes(service: &NoteService) -> Response {
    match service.get_all_notes().await {
        Ok(notes) => {
            let notes_xml: Vec<NoteResponseXml> = notes
                .into_iter()
                .map(|note| NoteResponseXml {
                    id: note.id,
                    content: note.content,
                })
                .collect();

            let response = GetAllNotesResponse {
                m_ns: "https://notes-server/soap/v1".to_string(),
                notes: notes_xml,
            };

            let envelope = GetAllNotesEnvelope {
                soap_ns: "http://www.w3.org/2003/05/soap-envelope".to_string(),
                encoding_style: "http://www.w3.org/2003/05/soap-encoding".to_string(),
                body: GetAllNotesBody { response },
            };

            let xml_body = match quick_xml::se::to_string(&envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&format!("{e}")),
            };

            build_ok_response(xml_body)
        }
        Err(e) => handle_internal_error(&e, "Failed to get note"),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "soap:Envelope")]
struct UpdateNoteEnvelope {
    #[serde(rename = "@xmlns:soap")]
    soap_ns: String,
    #[serde(rename = "@soap:encodingStyle")]
    encoding_style: String,
    #[serde(rename = "soap:Body")]
    body: UpdateNoteBody,
}

#[derive(Debug, Serialize)]
struct UpdateNoteBody {
    #[serde(rename = "m:UpdateNoteResponse")]
    response: UpdateNoteResponse,
}

async fn handle_update_note(service: &NoteService, req: UpdateNoteRequest) -> Response {
    let dto_req = dto::UpdateNoteRequest {
        content: req.content,
    };

    match service.update_note(req.id, dto_req).await {
        Ok(Some(note)) => {
            let response = UpdateNoteResponse {
                m_ns: "https://notes-server/soap/v1".to_string(),
                note: NoteResponseXml {
                    id: note.id,
                    content: note.content,
                },
            };

            let envelope = UpdateNoteEnvelope {
                soap_ns: "http://www.w3.org/2003/05/soap-envelope".to_string(),
                encoding_style: "http://www.w3.org/2003/05/soap-encoding".to_string(),
                body: UpdateNoteBody { response },
            };

            let xml_body = match quick_xml::se::to_string(&envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&format!("{e}")),
            };

            build_ok_response(xml_body)
        }
        Ok(None) => handle_not_found_error(),
        Err(e) => handle_internal_error(&e, "Failed to update note"),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "soap:Envelope")]
struct DeleteNoteEnvelope {
    #[serde(rename = "@xmlns:soap")]
    soap_ns: String,
    #[serde(rename = "@soap:encodingStyle")]
    encoding_style: String,
    #[serde(rename = "soap:Body")]
    body: DeleteNoteBody,
}

#[derive(Debug, Serialize)]
struct DeleteNoteBody {
    #[serde(rename = "m:DeleteNoteResponse")]
    response: DeleteNoteResponse,
}

async fn handle_delete_note(service: &NoteService, req: DeleteNoteRequest) -> Response {
    match service.delete_note(req.id).await {
        Ok(true) => {
            let response = DeleteNoteResponse {
                m_ns: "https://notes-server/soap/v1".to_string(),
            };

            let envelope = DeleteNoteEnvelope {
                soap_ns: "http://www.w3.org/2003/05/soap-envelope".to_string(),
                encoding_style: "http://www.w3.org/2003/05/soap-encoding".to_string(),
                body: DeleteNoteBody { response },
            };

            let xml_body = match quick_xml::se::to_string(&envelope) {
                Ok(s) => s,
                Err(e) => return handle_serialization_error(&format!("{e}")),
            };

            build_ok_response(xml_body)
        }
        Ok(false) => handle_not_found_error(),
        Err(e) => handle_internal_error(&e, "Failed to delete note"),
    }
}
