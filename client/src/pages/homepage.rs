use dioxus::prelude::*;
use game::tet::GameState;

use crate::comp::{bot_player::BotPlayer, game_display::GameDisplay};
use crate::route::Route;
use crate::network::GlobalChatClientContext;

/// Home page with game preview and CTAs
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
        article { class: "hero-section",
            // Game Preview Section
            div { class: "game-preview",
                div { class: "game-preview-header",
                    "🎮 Live Preview"
                }
                div { style: "padding:1rem; background: rgba(0,0,0,0.2);",
                    BotPlayer {game_state}
                    div { style: "height: 60dvh; display: flex; justify-content: center; align-items: center;",
                        GameDisplay { game_state }
                    }
                }
            }
            
            // CTA Buttons
            div { class: "cta-section",
                Link {
                    class: "cta-button cta-button-primary",
                    to: Route::PlaySingleplayerPage {},
                    "🎯 Play Singleplayer"
                }
                Link {
                    class: "cta-button cta-button-secondary",
                    to: Route::MatchmakingPage {},
                    "⚔️ 1v1 Matchmaking"
                }
            }
            
            // Stats Bar
            div { class: "stats-bar",
                div { class: "stat-item",
                    div { class: "stat-value", "{online_count}" }
                    div { class: "stat-label", "Players Online" }
                }
            }
            
            // Quick Links
            div { class: "quick-links",
                Link {
                    class: "quick-link",
                    to: Route::GlobalChatPage {},
                    "💬 Chat"
                }
                Link {
                    class: "quick-link",
                    to: Route::UsersRootDirectoryPage {},
                    "🏆 Top Players"
                }
            }
        }
    }
}
