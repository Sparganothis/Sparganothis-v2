use dioxus::prelude::*;
use game::tet::GameState;

use crate::comp::{bot_player::BotPlayer, game_display::GameDisplay};
use crate::route::Route;
use crate::network::GlobalChatClientContext;

/// Home page with premium dashboard layout
#[component]
pub fn Home() -> Element {
    let game_state = use_signal(GameState::new_random);
    
    // Get chat context for online player count
    let chat_context = use_context::<GlobalChatClientContext>();
    let presence = chat_context.chat.presence;
    
    // Get online player count from presence list
    let online_count = use_memo(move || {
        let p = presence.read();
        p.0.len()
    });

    rsx! {
        div { class: "container",
            div { class: "dashboard-grid",
                
                // LEFT COLUMN: Game Modes
                div { class: "sidebar-left",
                    h2 { class: "mb-1", style: "font-size: 2rem;", "Play Your Way" }
                    p { class: "text-muted mb-2", "Choose your mode and start playing" }
                    
                    div { class: "card game-mode-card mode-singleplayer",
                        div { class: "mode-icon", "👤" }
                        h3 { "Singleplayer" }
                        p { "Challenge yourself and master the board." }
                        Link {
                            class: "btn btn-primary",
                            to: Route::PlaySingleplayerPage {},
                            "Play Singleplayer →"
                        }
                    }
                    
                    div { class: "card game-mode-card mode-matchmaking",
                        div { class: "mode-icon", "⚔️" }
                        h3 { "1v1 Matchmaking" }
                        p { "Compete against real players in real time." }
                        Link {
                            class: "btn btn-secondary",
                            to: Route::MatchmakingPage {},
                            "Find Match →"
                        }
                    }
                    
                    div { class: "card mt-2 flex items-center gap-1",
                        div { class: "online-indicator", "• {online_count} players online" }
                        span { class: "text-muted", style: "font-size: 0.8rem;", "Active community" }
                    }
                }
                
                // CENTER COLUMN: Last Game / Live Preview
                div { class: "main-content",
                    div { class: "card game-preview-card",
                        div { class: "card-header",
                            div { class: "card-title", 
                                "🕒 Last Game"
                            }
                            div { class: "status-badge status-victory", "Victory" }
                        }
                        
                        div { class: "game-display-container",
                            // Left Stats
                            div { class: "game-stats-list",
                                div { class: "game-stat-entry",
                                    span { class: "game-stat-label", "Score" }
                                    span { class: "game-stat-value", "920" }
                                }
                                div { class: "game-stat-entry",
                                    span { class: "game-stat-label", "Lines" }
                                    span { class: "game-stat-value", "12" }
                                }
                                div { class: "game-stat-entry",
                                    span { class: "game-stat-label", "Moves" }
                                    span { class: "game-stat-value", "153" }
                                }
                            }
                            
                            // Center Board
                            div { class: "flex flex-col items-center",
                                BotPlayer { game_state }
                                GameDisplay { game_state }
                            }
                            
                            // Right Stats (Next/Hold)
                            div { class: "game-stats-list",
                                div { class: "game-stat-entry",
                                    span { class: "game-stat-label", "Time" }
                                    span { class: "game-stat-value", "63.75s" }
                                }
                                div { class: "game-stat-entry",
                                    span { class: "game-stat-label", "Combo" }
                                    span { class: "game-stat-value text-victory", "-1" }
                                }
                            }
                        }
                        
                        div { class: "mt-2 text-center",
                            Link {
                                class: "btn btn-outline",
                                to: Route::UsersRootDirectoryPage {},
                                "View Full Game Stats →"
                            }
                        }
                    }
                }
                
                // RIGHT COLUMN: Chat & Top Players
                div { class: "sidebar-right",
                    
                    // Live Chat Section
                    div { class: "card sidebar-section",
                        div { class: "card-title mb-2", "💬 Live Chat" }
                        div { class: "chat-message",
                            div { class: "chat-avatar", style: "background: #4caf50;" }
                            div { class: "chat-content",
                                span { class: "user", "PlayerOne" }
                                span { class: "time", "2m ago" }
                                p { class: "mb-0", "GG!" }
                            }
                        }
                        div { class: "chat-message",
                            div { class: "chat-avatar", style: "background: #2196f3;" }
                            div { class: "chat-content",
                                span { class: "user", "TetrisMaster" }
                                span { class: "time", "5m ago" }
                                p { class: "mb-0", "Nice combo!" }
                            }
                        }
                        Link {
                            class: "btn btn-outline mt-1",
                            to: Route::GlobalChatPage {},
                            "Open Chat"
                        }
                    }
                    
                    // Top Players Section
                    div { class: "card sidebar-section",
                        div { class: "card-title mb-2", "🏆 Top Players" }
                        // Placeholder for top players list
                        div { class: "flex flex-col gap-1",
                            for (i, name) in ["TetrisMaster", "BlockKing", "SpeedDemon"].iter().enumerate() {
                                div { class: "flex justify-between items-center",
                                    span { "{i+1}. {name}" }
                                    span { class: "text-victory", "{2847 - (i as i32 * 500)}" }
                                }
                            }
                        }
                        Link {
                            class: "btn btn-outline mt-1",
                            to: Route::UsersRootDirectoryPage {},
                            "View Leaderboard"
                        }
                    }
                }

            }
            
            footer {
                p { "© 2024 Sparganothis. Built with Rust 🦀" }
            }
        }
    }
}

