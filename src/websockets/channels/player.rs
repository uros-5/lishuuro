// async fn player_task(
//     mut socket: WebSocket,
//     player: Player,
//     wsstate: WsState,
//     route: String,
// ) -> Sender<WsMessage> {
//     let (mut sender, mut receiver) = socket.split();
//     let (player_sender, mut player_receiver) = mpsc::channel::<WsMessage>(32);
//     let player_channel = player_sender.clone();
//     let mut current_room = CurrentRoom::from(route);
//     let game_requests = wsstate.game_requests.clone();
//     let players = wsstate.players.clone();
//     let tv = wsstate.tv.clone();
//     let mut current_game: Option<mpsc::Sender<GameMessage>> = None;
//     let games = wsstate.games.clone();
//     let send_task = tokio::spawn(async move {
//         while let Some(message) = player_receiver.recv().await {}
//     });

//     let receive_message_task = tokio::spawn(async move {
//         while let Some(message) = receiver.next().await {
//             if let Ok(message) = message {
//                 match message {
//                     Message::Text(content) => {
//                         if let Ok(message) =
//                             serde_json::from_str::<ClientMessage>(&content)
//                         {
//                             match message.t {
//                                 MessageType::ChangeRoom => {
//                                     if let Ok(message) =
//                                         serde_json::from_value::<String>(message.d)
//                                     {
//                                         let new_room = CurrentRoom::from(message);
//                                         if current_room != new_room {
//                                             current_room
//                                                 .leave(
//                                                     &player,
//                                                     &game_requests,
//                                                     &players,
//                                                     &current_game,
//                                                     &tv,
//                                                     false,
//                                                 )
//                                                 .await;
//                                             current_room = new_room;
//                                             match current_room {
//                                                 CurrentRoom::NoRoom => {}
//                                                 CurrentRoom::Home => {
//                                                     let _ = game_requests
//                                                         .send(
//                                                             GameRequestMessage::Join(
//                                                                 player
//                                                                     ._id
//                                                                     .to_string(),
//                                                                 player_channel
//                                                                     .clone(),
//                                                             ),
//                                                         )
//                                                         .await;

//                                                     let _ = players
//                                                         .send(PlayersMessage::Join(
//                                                             player._id.to_string(),
//                                                             player_channel.clone(),
//                                                         ))
//                                                         .await;
//                                                 }
//                                                 CurrentRoom::Tv => {
//                                                     let _ = tv
//                                                         .send(TvMessage::Join(
//                                                             player._id.to_string(),
//                                                             player_channel.clone(),
//                                                         ))
//                                                         .await;
//                                                 }
//                                                 CurrentRoom::Game(_) => {
//                                                     let (sender, receiver) =
//                                                         oneshot::channel();
//                                                     let _ = games.send(
//                                                         GamesMessage::GetChannel {
//                                                             sender,
//                                                             player: player_channel.clone()
//                                                         },
//                                                     ).await;
//                                                     if let Ok(game) = receiver.await
//                                                     {
//                                                         current_game = Some(game);
//                                                     } else {
//                                                         current_game = None;
//                                                     }
//                                                 }
//                                                 CurrentRoom::OtherRoom => {}
//                                             }
//                                         }
//                                     }
//                                 }

//                                 MessageType::AddGameRequest => {
//                                     if current_room == CurrentRoom::Home {
//                                         // if let Ok(game_request) =
//                                         //     serde_json::from_value(message.d)
//                                         // {
//                                         //     let _ = game_requests.send(
//                                         //         GameRequestMessage::AddGameRequest(
//                                         //             player._id.to_string(),
//                                         //             game_request,

//                                         //         ),
//                                         //     ).await;
//                                         // }
//                                     }
//                                 }
//                                 MessageType::AcceptGameRequest => {
//                                     if current_room == CurrentRoom::Home {
//                                         if let Ok(wait) =
//                                             serde_json::from_value(message.d)
//                                         {
//                                             let message = GameRequestMessage::AcceptGameRequest { wait,
//                                                  accept: player._id.to_string()};
//                                             let _ =
//                                                 game_requests.send(message).await;
//                                         }
//                                     }
//                                 }
//                                 MessageType::PlayerCount => {
//                                     if current_room == CurrentRoom::Home {
//                                         let _ = players
//                                             .send(PlayersMessage::IncrementCount(
//                                                 player._id.to_string(),
//                                             ))
//                                             .await;
//                                     }
//                                 }
//                                 MessageType::GameCount => {
//                                     if current_room == CurrentRoom::Home {
//                                         let _ = games
//                                             .send(GamesMessage::Count(
//                                                 player._id.to_string(),
//                                             ))
//                                             .await;
//                                     }
//                                 }
//                                 MessageType::GetGame => {}
//                                 MessageType::GetHistory => {
//                                     if let Some(ref game) = current_game {
//                                         if let Ok(stage) =
//                                             serde_json::from_value(message.d)
//                                         {
//                                             let _ = game
//                                                 .send(GameMessage::GetHistory(
//                                                     player._id.to_string(),
//                                                     stage,
//                                                 ))
//                                                 .await;
//                                         }
//                                     }
//                                 }
//                                 MessageType::GetHand => {
//                                     if let Some(ref game) = current_game {
//                                         let _ = game
//                                             .send(GameMessage::GetHand(
//                                                 player._id.to_string(),
//                                             ))
//                                             .await;
//                                     }
//                                 }
//                                 MessageType::SelectMove => {
//                                     if let Some(ref game) = current_game {
//                                         if let Ok(selected) =
//                                             serde_json::from_value(message.d)
//                                         {
//                                             let _ = game
//                                                 .send(GameMessage::SelectMove {
//                                                     player: player._id.to_string(),
//                                                     game_move: selected,
//                                                 })
//                                                 .await;
//                                         }
//                                     }
//                                 }
//                                 MessageType::PlacePiece => {
//                                     if let Some(ref game) = current_game {
//                                         if let Ok(placed) =
//                                             serde_json::from_value(message.d)
//                                         {
//                                             let _ = game
//                                                 .send(GameMessage::PlacePiece {
//                                                     player: player._id.to_string(),
//                                                     game_move: placed,
//                                                 })
//                                                 .await;
//                                         }
//                                     }
//                                 }
//                                 MessageType::MovePiece => {
//                                     if let Some(ref game) = current_game {
//                                         if let Ok(moved) =
//                                             serde_json::from_value(message.d)
//                                         {
//                                             let _ = game
//                                                 .send(GameMessage::MovePiece {
//                                                     player: player._id.to_string(),
//                                                     game_move: moved,
//                                                 })
//                                                 .await;
//                                         }
//                                     }
//                                 }
//                                 MessageType::Draw => {
//                                     if let Some(ref game) = current_game {
//                                         let _ = game
//                                             .send(GameMessage::Draw(
//                                                 player._id.to_string(),
//                                             ))
//                                             .await;
//                                     }
//                                 }
//                                 MessageType::Resign => {
//                                     if let Some(ref game) = current_game {
//                                         let _ = game
//                                             .send(GameMessage::Resign(
//                                                 player._id.to_string(),
//                                             ))
//                                             .await;
//                                     }
//                                 }
//                                 MessageType::GetTv => {
//                                     if current_room == CurrentRoom::Tv {
//                                         let _ = tv
//                                             .send(TvMessage::Join(
//                                                 player._id.to_string(),
//                                                 player_channel.clone(),
//                                             ))
//                                             .await;
//                                     }
//                                 }
//                                 MessageType::SaveState => {
//                                     if &player._id.to_string() == "iiiurosiii" {}
//                                 }
//                                 MessageType::ReloadJinja => {}
//                                 _ => {}
//                             }
//                         }

//                         // listening for client messages
//                     }
//                     Message::Close(close) => {
//                         let _ = players
//                             .send(PlayersMessage::Leave {
//                                 player: player._id.to_string(),
//                                 disconnected: true,
//                             })
//                             .await;

//                         current_room
//                             .leave(
//                                 &player,
//                                 &game_requests,
//                                 &players,
//                                 &current_game,
//                                 &tv,
//                                 true,
//                             )
//                             .await;
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     });
//     player_sender
// }
