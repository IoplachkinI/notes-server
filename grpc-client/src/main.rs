use tonic::Request;

// Include the generated proto code
pub mod notes {
    tonic::include_proto!("notes");
}

use notes::{
    CreateNoteRequest, DeleteNoteRequest, GetAllNotesRequest, GetNoteRequest, UpdateNoteRequest,
    note_service_client::NoteServiceClient,
};

use serde_json::to_string_pretty;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the gRPC server
    let addr =
        std::env::var("GRPC_SERVER_ADDR").unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    let mut client = NoteServiceClient::connect(addr.clone()).await?;
    println!("Connected to gRPC server at address {}\n", addr);

    // Create note
    println!("1. Creating a note...");
    let create_request = CreateNoteRequest {
        content: "Test string gRPC".to_string(),
    };
    let create_response = client.create_note(Request::new(create_request)).await?;
    let created_note = create_response.into_inner();
    println!("Created note: {}\n", to_string_pretty(&created_note)?);
    let note_id = created_note.id;

    // Get one note
    println!("2. Getting note by ID...");
    let get_request = GetNoteRequest { id: note_id };
    let get_response = client.get_note(Request::new(get_request)).await?;
    let note = get_response.into_inner();
    println!("Note: {}\n", to_string_pretty(&note)?);

    // Update note
    println!("3. Updating the note...");
    let update_request = UpdateNoteRequest {
        id: note_id,
        content: "Test string gRPC 2".to_string(),
    };
    let update_response = client.update_note(Request::new(update_request)).await?;
    let updated_note = update_response.into_inner();
    println!("Updated note: {}\n", to_string_pretty(&updated_note)?);

    // Get all notes
    println!("4. Getting all notes...");
    let get_all_request = GetAllNotesRequest {};
    let get_all_response = client.get_all_notes(Request::new(get_all_request)).await?;
    let all_notes = get_all_response.into_inner();
    println!("Notes: {}\n", to_string_pretty(&all_notes)?);

    // Delete note
    println!("5. Deleting the note...");
    let delete_request = DeleteNoteRequest { id: note_id };
    let delete_response = client.delete_note(Request::new(delete_request)).await?;
    let delete_result = delete_response.into_inner();
    println!("Delete result: {}\n", delete_result.success);

    Ok(())
}
