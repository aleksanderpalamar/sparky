use iced::Task;
use iced::futures::StreamExt;
use iced_runtime::Action as TaskAction;

use super::message::Message;
use super::state::Files;
use super::update::update;

pub fn concluir(files: &mut Files, task: Task<Message>, limite: usize) -> usize {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut queue = vec![task];
    let mut processadas = 0;
    while let Some(task) = queue.pop() {
        let Some(mut stream) = iced_runtime::task::into_stream(task) else {
            continue;
        };
        let messages = runtime.block_on(async {
            let mut messages = Vec::new();
            while let Some(action) = stream.next().await {
                if let TaskAction::Output(message) = action {
                    messages.push(message);
                }
            }
            messages
        });
        for message in messages {
            processadas += 1;
            if processadas > limite {
                return processadas;
            }
            queue.push(update(files, message));
        }
    }
    processadas
}
