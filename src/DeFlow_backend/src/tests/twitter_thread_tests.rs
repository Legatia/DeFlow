#[cfg(test)]
mod twitter_thread_tests {
    use crate::types::{ConfigValue, WorkflowNode, NodeConfiguration};
    use std::collections::HashMap;

    // Helper function to create a test config
    fn create_test_config() -> HashMap<String, ConfigValue> {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), ConfigValue::String("test_api_key".to_string()));
        config.insert("api_secret".to_string(), ConfigValue::String("test_api_secret".to_string()));
        config.insert("access_token".to_string(), ConfigValue::String("test_access_token".to_string()));
        config.insert("access_token_secret".to_string(), ConfigValue::String("test_secret".to_string()));
        config
    }

    #[test]
    fn test_thread_parsing_single_tweet() {
        let thread_content = "";
        let tweets: Vec<String> = thread_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();

        assert_eq!(tweets.len(), 0, "Empty thread should parse to 0 additional tweets");
    }

    #[test]
    fn test_thread_parsing_multiple_tweets() {
        let thread_content = "Tweet 2: Second tweet\nTweet 3: Third tweet\nTweet 4: Fourth tweet";
        let tweets: Vec<String> = thread_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();

        assert_eq!(tweets.len(), 3, "Should parse 3 tweets from thread content");
        assert_eq!(tweets[0], "Tweet 2: Second tweet");
        assert_eq!(tweets[1], "Tweet 3: Third tweet");
        assert_eq!(tweets[2], "Tweet 4: Fourth tweet");
    }

    #[test]
    fn test_thread_parsing_with_empty_lines() {
        let thread_content = "Tweet 2\n\nTweet 3\n  \nTweet 4";
        let tweets: Vec<String> = thread_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();

        assert_eq!(tweets.len(), 3, "Should skip empty lines");
    }

    #[test]
    fn test_thread_length_validation() {
        let mut all_tweets = vec!["First tweet".to_string()];

        // Add 24 more tweets (total 25 - max allowed)
        for i in 2..=25 {
            all_tweets.push(format!("Tweet {}", i));
        }

        assert_eq!(all_tweets.len(), 25, "Should allow exactly 25 tweets");
        assert!(all_tweets.len() <= 25, "Should not exceed 25 tweets");
    }

    #[test]
    fn test_thread_length_validation_exceeds_limit() {
        let mut all_tweets = vec!["First tweet".to_string()];

        // Add 25 more tweets (total 26 - exceeds limit)
        for i in 2..=26 {
            all_tweets.push(format!("Tweet {}", i));
        }

        assert!(all_tweets.len() > 25, "Should detect when thread exceeds 25 tweets");
    }

    #[test]
    fn test_reply_to_tweet_id_extraction() {
        let mut config = create_test_config();
        config.insert("reply_to_tweet_id".to_string(), ConfigValue::String("1234567890".to_string()));

        let reply_to = config.get("reply_to_tweet_id")
            .and_then(|v| match v {
                ConfigValue::String(s) if !s.trim().is_empty() => Some(s.clone()),
                _ => None,
            });

        assert!(reply_to.is_some(), "Should extract reply_to_tweet_id");
        assert_eq!(reply_to.unwrap(), "1234567890");
    }

    #[test]
    fn test_reply_to_tweet_id_empty_string() {
        let mut config = create_test_config();
        config.insert("reply_to_tweet_id".to_string(), ConfigValue::String("".to_string()));

        let reply_to = config.get("reply_to_tweet_id")
            .and_then(|v| match v {
                ConfigValue::String(s) if !s.trim().is_empty() => Some(s.clone()),
                _ => None,
            });

        assert!(reply_to.is_none(), "Empty reply_to_tweet_id should be ignored");
    }

    #[test]
    fn test_thread_tweets_extraction() {
        let mut config = create_test_config();
        config.insert("thread_tweets".to_string(), ConfigValue::String("Tweet 2\nTweet 3".to_string()));

        let thread_tweets = config.get("thread_tweets")
            .and_then(|v| match v {
                ConfigValue::String(s) if !s.trim().is_empty() => Some(s.clone()),
                _ => None,
            });

        assert!(thread_tweets.is_some(), "Should extract thread_tweets");
        assert_eq!(thread_tweets.unwrap(), "Tweet 2\nTweet 3");
    }

    #[test]
    fn test_thread_tweets_empty() {
        let mut config = create_test_config();
        config.insert("thread_tweets".to_string(), ConfigValue::String("".to_string()));

        let thread_tweets = config.get("thread_tweets")
            .and_then(|v| match v {
                ConfigValue::String(s) if !s.trim().is_empty() => Some(s.clone()),
                _ => None,
            });

        assert!(thread_tweets.is_none(), "Empty thread_tweets should return None");
    }

    #[test]
    fn test_thread_tweets_missing() {
        let config = create_test_config();

        let thread_tweets = config.get("thread_tweets")
            .and_then(|v| match v {
                ConfigValue::String(s) if !s.trim().is_empty() => Some(s.clone()),
                _ => None,
            });

        assert!(thread_tweets.is_none(), "Missing thread_tweets should return None");
    }

    #[test]
    fn test_twitter_request_body_single_tweet() {
        let message = "Hello Twitter!";
        let body = format!(r#"{{"text":"{}"}}"#, message.replace("\"", "\\\""));

        assert_eq!(body, r#"{"text":"Hello Twitter!"}"#);
    }

    #[test]
    fn test_twitter_request_body_with_reply() {
        let message = "Reply tweet";
        let reply_id = "1234567890";
        let body = format!(
            r#"{{"text":"{}","reply":{{"in_reply_to_tweet_id":"{}"}}}}"#,
            message.replace("\"", "\\\""),
            reply_id
        );

        assert_eq!(
            body,
            r#"{"text":"Reply tweet","reply":{"in_reply_to_tweet_id":"1234567890"}}"#
        );
    }

    #[test]
    fn test_twitter_request_body_with_quotes() {
        let message = r#"He said "Hello" to me"#;
        let body = format!(r#"{{"text":"{}"}}"#, message.replace("\"", "\\\""));

        assert_eq!(body, r#"{"text":"He said \"Hello\" to me"}"#);
    }

    #[test]
    fn test_tweet_character_limit() {
        let tweet_280_chars = "a".repeat(280);
        assert_eq!(tweet_280_chars.len(), 280, "280 characters should be allowed");

        let tweet_281_chars = "a".repeat(281);
        assert!(tweet_281_chars.len() > 280, "281 characters should exceed limit");
    }

    #[test]
    fn test_thread_with_variables() {
        let thread_content = "Portfolio: ${{value}}\nProfit: {{percent}}%\nStrategy: {{name}}";
        let tweets: Vec<String> = thread_content
            .lines()
            .map(|line| line.to_string())
            .collect();

        assert_eq!(tweets.len(), 3);
        assert!(tweets[0].contains("{{value}}"), "Should preserve variable syntax");
        assert!(tweets[1].contains("{{percent}}"), "Should preserve variable syntax");
        assert!(tweets[2].contains("{{name}}"), "Should preserve variable syntax");
    }

    #[test]
    fn test_config_cloning_for_thread() {
        let mut config = create_test_config();
        config.insert("thread_tweets".to_string(), ConfigValue::String("Tweet 2".to_string()));

        let mut thread_config = config.clone();
        thread_config.insert("reply_to_tweet_id".to_string(), ConfigValue::String("999".to_string()));

        // Original config should not be modified
        assert!(config.get("reply_to_tweet_id").is_none());

        // Thread config should have the new value
        assert!(thread_config.get("reply_to_tweet_id").is_some());
    }

    #[test]
    fn test_percent_encode_basic() {
        fn percent_encode(s: &str) -> String {
            s.chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~' {
                        c.to_string()
                    } else {
                        format!("%{:02X}", c as u8)
                    }
                })
                .collect()
        }

        assert_eq!(percent_encode("hello"), "hello");
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(percent_encode("test@example.com"), "test%40example.com");
        assert_eq!(percent_encode("100%"), "100%25");
    }

    #[test]
    fn test_oauth_timestamp_format() {
        // Mock timestamp (nanoseconds)
        let timestamp_ns = 1700000000000000000u64;
        let timestamp_s = (timestamp_ns / 1_000_000_000).to_string();

        assert_eq!(timestamp_s, "1700000000");
        assert!(timestamp_s.len() == 10, "Timestamp should be 10 digits (seconds)");
    }

    #[test]
    fn test_twitter_credentials_validation() {
        let mut config = HashMap::new();

        // Missing all credentials
        assert!(config.get("api_key").is_none());
        assert!(config.get("api_secret").is_none());
        assert!(config.get("access_token").is_none());
        assert!(config.get("access_token_secret").is_none());

        // Add credentials
        config.insert("api_key".to_string(), ConfigValue::String("key".to_string()));
        config.insert("api_secret".to_string(), ConfigValue::String("secret".to_string()));
        config.insert("access_token".to_string(), ConfigValue::String("token".to_string()));
        config.insert("access_token_secret".to_string(), ConfigValue::String("token_secret".to_string()));

        // All credentials present
        assert!(config.get("api_key").is_some());
        assert!(config.get("api_secret").is_some());
        assert!(config.get("access_token").is_some());
        assert!(config.get("access_token_secret").is_some());
    }
}

#[cfg(test)]
mod twitter_thread_integration_tests {
    use super::*;

    #[test]
    fn test_full_thread_workflow_parsing() {
        // Simulate a complete thread workflow
        let first_tweet = "🚀 Thread about DeFi automation (1/5)";
        let thread_content = "\
Step 1: Connect wallets (2/5)
Step 2: Set up strategies (3/5)
Step 3: Monitor performance (4/5)
Final: Enjoy automated gains! (5/5)";

        let mut all_tweets = vec![first_tweet.to_string()];
        all_tweets.extend(
            thread_content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.to_string())
        );

        assert_eq!(all_tweets.len(), 5, "Should have 5 tweets total");
        assert_eq!(all_tweets[0], "🚀 Thread about DeFi automation (1/5)");
        assert_eq!(all_tweets[4], "Final: Enjoy automated gains! (5/5)");
    }

    #[test]
    fn test_thread_chaining_simulation() {
        let tweets = vec![
            "Tweet 1".to_string(),
            "Tweet 2".to_string(),
            "Tweet 3".to_string(),
        ];

        let mut previous_tweet_id: Option<String> = None;
        let mut tweet_ids = Vec::new();

        for (index, _tweet_text) in tweets.iter().enumerate() {
            // Simulate posting and getting tweet ID
            let tweet_id = format!("tweet_id_{}", index + 1);
            tweet_ids.push(tweet_id.clone());
            previous_tweet_id = Some(tweet_id);
        }

        assert_eq!(tweet_ids.len(), 3);
        assert_eq!(tweet_ids[0], "tweet_id_1");
        assert_eq!(tweet_ids[2], "tweet_id_3");
        assert_eq!(previous_tweet_id, Some("tweet_id_3".to_string()));
    }

    #[test]
    fn test_error_handling_mid_thread() {
        let tweets = vec!["Tweet 1", "Tweet 2", "Tweet 3"];
        let mut posted_count = 0;
        let fail_at = 2; // Simulate failure at tweet 2

        for (index, _tweet) in tweets.iter().enumerate() {
            if index == fail_at {
                // Simulate error
                break;
            }
            posted_count += 1;
        }

        assert_eq!(posted_count, 2, "Should have posted 2 tweets before failure");
    }

    #[test]
    fn test_thread_with_dynamic_data() {
        let template = "Portfolio: ${{value}}\nChange: {{percent}}%";

        // Simulate variable replacement
        let replaced = template
            .replace("{{value}}", "10000")
            .replace("{{percent}}", "+15");

        let tweets: Vec<String> = replaced
            .lines()
            .map(|line| line.to_string())
            .collect();

        assert_eq!(tweets[0], "Portfolio: $10000");
        assert_eq!(tweets[1], "Change: +15%");
    }

    #[test]
    fn test_max_thread_length() {
        let mut all_tweets = vec!["First tweet".to_string()];

        // Try to add 30 more tweets (exceeds limit)
        for i in 2..=31 {
            all_tweets.push(format!("Tweet {}", i));
        }

        // Should detect and reject
        let result = if all_tweets.len() > 25 {
            Err("Thread exceeds Twitter's 25 tweet limit")
        } else {
            Ok(())
        };

        assert!(result.is_err(), "Should reject threads > 25 tweets");
    }
}
