# Social Media Integration Tests

This document provides comprehensive information about testing the Telegram and Discord integrations in the DeFlow backend system.

## Overview

The DeFlow backend includes social media integration capabilities for sending notifications and alerts to users via Telegram and Discord. This test suite validates the functionality of these integrations.

## Test Structure

### Test Modules

1. **`social_media_basic_tests.rs`** - Core functionality tests for social media integrations
2. **`tests/mod.rs`** - Test module organization and utilities

### Key Test Areas

#### 1. Social Platform Enumeration Tests
- Validates that `SocialPlatform::Telegram` and `SocialPlatform::Discord` are properly defined
- Tests pattern matching for platform selection
- Ensures platform enums work correctly in match statements

#### 2. Price Alert Integration Tests
- Tests price alert structures with social media actions
- Validates `AlertAction::SocialPost` configurations
- Tests multi-platform alert setups (Telegram + Discord)
- Verifies message template handling

#### 3. Price Condition Tests
- Tests `PriceCondition::Above` and `PriceCondition::Below` conditions
- Validates price threshold logic
- Tests condition matching and pattern handling

#### 4. Token Price Structure Tests
- Validates `TokenPrice` data structure
- Tests price data handling for alerts
- Verifies timestamp and volume data

#### 5. Message Template Tests
- Tests variable substitution patterns (${price}, ${symbol}, etc.)
- Validates platform-specific formatting
- Tests HTML/Markdown formatting for different platforms

#### 6. Alert Validation Tests
- Tests valid and invalid alert configurations
- Validates required fields (id, user_id, token_symbol, etc.)
- Tests edge cases (empty symbols, no conditions, no actions)

#### 7. Platform-Specific Configuration Tests
- Tests Telegram-specific features (HTML formatting, bot tokens)
- Tests Discord-specific features (webhooks, embed formatting)
- Validates platform-specific message formatting

#### 8. Alert State Management Tests
- Tests alert activation/deactivation
- Validates last_triggered timestamp handling
- Tests alert lifecycle management

## Running the Tests

### Prerequisites

1. Ensure Rust and Cargo are installed
2. Navigate to the backend directory:
   ```bash
   cd /Users/zhang/Desktop/ICP/DeFlow/src/DeFlow_backend
   ```

### Running All Tests

```bash
# Run all tests
cargo test

# Run only social media tests
cargo test social_media_basic_tests

# Run tests with output
cargo test social_media_basic_tests -- --nocapture
```

### Running Specific Test Functions

```bash
# Test social platform enums
cargo test test_social_platform_enum

# Test price alerts with social actions
cargo test test_price_alert_with_social_actions

# Test message templates
cargo test test_message_template_patterns

# Test platform-specific configurations
cargo test test_platform_specific_configs
```

### Test Utilities

The test suite includes utility functions for creating test data:

```rust
use crate::tests::{
    create_test_telegram_alert,
    create_test_discord_alert,
    create_test_multi_platform_alert,
    create_test_token_price,
    validate_alert_structure,
};
```

#### Utility Functions

- **`create_test_telegram_alert()`** - Creates a sample Telegram alert
- **`create_test_discord_alert()`** - Creates a sample Discord alert
- **`create_test_multi_platform_alert()`** - Creates an alert for both platforms
- **`create_test_token_price(symbol, price)`** - Creates test price data
- **`validate_alert_structure(alert)`** - Validates alert structure

## Test Coverage

### Current Test Coverage

✅ **Social Platform Enums**
- Telegram platform definition
- Discord platform definition
- Pattern matching

✅ **Price Alert Structure**
- Alert creation and validation
- Social action configuration
- Multi-platform setup

✅ **Price Conditions**
- Above threshold conditions
- Below threshold conditions
- Condition matching

✅ **Token Price Handling**
- Price data structure
- Timestamp validation
- Volume and change data

✅ **Message Templates**
- Variable substitution patterns
- Platform-specific formatting
- Template validation

✅ **Alert Validation**
- Required field validation
- Edge case handling
- State management

### Areas for Future Enhancement

🔄 **Integration Tests**
- End-to-end workflow testing
- HTTP outcall mocking
- Rate limiting validation

🔄 **Error Handling Tests**
- Network failure scenarios
- Invalid token/webhook handling
- Rate limiting edge cases

🔄 **Performance Tests**
- High-volume alert processing
- Concurrent message sending
- Memory usage validation

## Mock Data and Test Environment

### Test Data Patterns

The tests use realistic mock data that reflects production scenarios:

```rust
// Example test alert
PriceAlert {
    id: "test_alert_123",
    user_id: "test_user",
    token_symbol: "BTC",
    conditions: vec![PriceCondition::Above(50000.0)],
    actions: vec![AlertAction::SocialPost {
        platforms: vec![SocialPlatform::Telegram],
        message_template: "🚀 BTC reached ${price}!",
    }],
    is_active: true,
    created_at: 1640995200,
    last_triggered: None,
}
```

### Message Template Examples

**Telegram Templates:**
- Simple: `"${symbol} price: ${price}"`
- HTML: `"<b>${symbol}</b> is now <i>${price}</i>"`
- Emoji: `"🚀 ${symbol} 📈 ${price}"`

**Discord Templates:**
- Embed: `"**${symbol} Alert**\n\nPrice: ${price}"`
- Mention: `"@here ${symbol} price is now ${price}!"`
- Rich: `"🔔 **${symbol}** reached ${price} with ${change}% change"`

## Debugging Tests

### Common Issues and Solutions

1. **Import Errors**
   ```bash
   error[E0432]: unresolved import
   ```
   - Solution: Check that all imports reference existing modules and types

2. **Missing Types**
   ```bash
   error[E0433]: failed to resolve: could not find `Type` in module
   ```
   - Solution: Verify that the type is properly exported from its module

3. **Pattern Matching Errors**
   ```bash
   error[E0004]: non-exhaustive patterns
   ```
   - Solution: Add missing match arms or use wildcard pattern

### Test Debugging Commands

```bash
# Run tests with detailed output
cargo test social_media_basic_tests -- --nocapture

# Run a specific test with debugging
RUST_LOG=debug cargo test test_social_platform_enum -- --nocapture

# Check compilation without running tests
cargo check --tests

# Run tests with timing information
cargo test social_media_basic_tests -- --nocapture --test-threads=1
```

## Integration with CI/CD

### GitHub Actions Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: Run Social Media Tests
  run: |
    cd src/DeFlow_backend
    cargo test social_media_basic_tests --verbose
```

### Test Performance Monitoring

```bash
# Measure test execution time
time cargo test social_media_basic_tests

# Profile test memory usage
cargo test social_media_basic_tests --release
```

## Contributing to Tests

### Adding New Tests

1. **Create test function:**
   ```rust
   #[test]
   fn test_new_functionality() {
       // Test implementation
       assert!(true);
   }
   ```

2. **Use existing utilities:**
   ```rust
   let alert = create_test_telegram_alert();
   assert!(validate_alert_structure(&alert));
   ```

3. **Follow naming conventions:**
   - `test_<functionality>_<specific_case>`
   - Use descriptive names that explain what's being tested

### Test Documentation Guidelines

- Include docstrings for complex test functions
- Explain test scenarios and expected outcomes
- Document any special setup or teardown requirements
- Reference related functionality in the main codebase

## Maintenance

### Regular Test Updates

1. **When adding new social platforms:**
   - Add platform enum tests
   - Add platform-specific formatting tests
   - Update multi-platform tests

2. **When modifying alert structure:**
   - Update validation tests
   - Verify backward compatibility
   - Update utility functions

3. **When changing message templates:**
   - Update template pattern tests
   - Verify variable substitution
   - Test platform-specific formatting

### Test Health Monitoring

```bash
# Check for unused test code
cargo test --unused

# Verify test coverage
cargo tarpaulin --out Html

# Run tests in different configurations
cargo test --features "test-mode"
```

## Summary

This test suite provides comprehensive coverage of the Telegram and Discord integration functionality in the DeFlow backend. The tests focus on core data structures, message formatting, and alert configuration validation.

For questions or issues with the social media tests, refer to:
- Test source code in `src/tests/social_media_basic_tests.rs`
- Utility functions in the `social_test_utilities` module
- Main social media integration code in `src/defi/price_alert_service.rs`

The tests are designed to be maintainable, readable, and provide confidence in the social media integration functionality across different scenarios and edge cases.