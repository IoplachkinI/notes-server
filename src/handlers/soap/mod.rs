use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{StatusCode, header::GetAll},
    response::{IntoResponse, Response},
};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{dto, service::NoteService};

// Request envelope

#[derive(Debug, YaDeserialize, YaSerialize)]
#[yaserde(rename = "Envelope")]
pub struct SoapEnvelope {
    #[yaserde(rename = "Body")]
    pub body: SoapBody,
}

// Request body

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct SoapBody {
    /// ``CreateNote`` operation request
    #[yaserde(rename = "CreateNote")]
    pub create: Option<CreateNoteRequest>,

    /// ``GetOneNote`` operation request
    #[yaserde(rename = "GetNote")]
    pub get_one: Option<GetOneNoteRequest>,

    /// ``GetAllNotes`` operation request
    #[yaserde(rename = "UpdateNote")]
    pub get_all: Option<GetAllNotesRequest>,

    /// ``UpdateNote`` operation request
    #[yaserde(rename = "UpdateNote")]
    pub update: Option<UpdateNoteRequest>,

    /// ``DeleteNote`` operation request
    #[yaserde(rename = "DeleteNote")]
    pub delete: Option<DeleteNoteRequest>,
}

// Request content variants

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct CreateNoteRequest {
    #[yaserde(rename = "Content")]
    pub content: String,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct GetOneNoteRequest {
    #[yaserde(rename = "Id")]
    pub id: i64,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct GetAllNotesRequest;

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct UpdateNoteRequest {
    #[yaserde(rename = "Id")]
    pub id: i64,

    #[yaserde(rename = "Content")]
    pub content: String,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct DeleteNoteRequest {
    #[yaserde(rename = "Id")]
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
    if let Some(_g) = body.get_one {
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

// CreateResponse

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct CreateNoteResponseEnvelope {
    #[yaserde(rename = "Body")]
    pub body: CreateNoteResponseBody,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct CreateNoteResponseBody {
    #[yaserde(rename = "CreateNoteResponse")]
    pub create_note_response: CreateNoteResponse,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct CreateNoteResponse {
    #[yaserde(rename = "Note")]
    pub note: NoteResponseXml,
}

#[derive(Debug, YaDeserialize, YaSerialize)]
pub struct NoteResponseXml {
    #[yaserde(rename = "Id")]
    pub id: i64,

    #[yaserde(rename = "Content")]
    pub content: String,
}

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
        // Some(NoteOperationRequest::GetAll) => handle_get_all_notes(&service).await,
        None => {
            let fault_xml = build_soap_fault(SoapFaultCode::Client, "Invalid SOAP request format");
            (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/xml; charset=utf-8")],
                fault_xml,
            )
                .into_response()
        }
    }

    // if let Some(create) = envelope.body.create {
    //     return handle_create_note(&service, create).await;
    // }

    // if let Some(get) = envelope.body.get_one {
    //     return handle_get_one_note(&service, get).await;
    // }
}

/// Common SOAP 1.1 fault codes.
#[derive(Debug, Clone, Copy)]
pub enum SoapFaultCode {
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
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Client => "Client",
            Self::Server => "Server",
            Self::MustUnderstand => "MustUnderstand",
            Self::VersionMismatch => "VersionMismatch",
        }
    }
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
                    create_note_response: CreateNoteResponse { note: note_xml },
                },
            };

            let xml_string = match yaserde::ser::to_string(&response_envelope) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to serialize SOAP response: {e}");
                    let fault_xml = build_soap_fault(
                        SoapFaultCode::Server,
                        "Failed to serialize SOAP response",
                    );
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [("Content-Type", "text/xml; charset=utf-8")],
                        fault_xml,
                    )
                        .into_response();
                }
            };

            (
                StatusCode::OK,
                [("Content-Type", "text/xml; charset=utf-8")],
                xml_string,
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create note via SOAP: {e}");
            // TODO: return SOAP Fault instead of plain text
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create note").into_response()
        }
    }
}

async fn handle_get_one_note(service: &NoteService, req: GetOneNoteRequest) -> Response {
    match service.get_one_note(req.id).await {
        Ok(Some(note)) => {
            let note_xml = NoteResponseXml {
                id: note.id,
                content: note.content,
            };

            // For brevity we reuse the same response envelope shape as CreateNoteResponse;
            // in a real API you might define a dedicated GetNoteResponseEnvelope.
            let response_envelope = CreateNoteResponseEnvelope {
                body: CreateNoteResponseBody {
                    create_note_response: CreateNoteResponse { note: note_xml },
                },
            };

            let xml_string = match yaserde::ser::to_string(&response_envelope) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to serialize SOAP response: {e}");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to serialize SOAP response",
                    )
                        .into_response();
                }
            };

            (
                StatusCode::OK,
                [("Content-Type", "text/xml; charset=utf-8")],
                xml_string,
            )
                .into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Note not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch note via SOAP: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch note").into_response()
        }
    }
}
