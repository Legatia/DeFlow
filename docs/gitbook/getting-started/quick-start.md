# Quick Start Guide

{% hint style="success" %}
Get up and running with DeFlow in under 10 minutes! This guide will walk you through creating your first workflow.
{% endhint %}

## 🎯 What You'll Build

In this tutorial, you'll create a simple portfolio monitoring workflow that:
1. Checks your DeFi portfolio balance
2. Sends a Discord notification if gains exceed 5%
3. Automatically runs daily at 9 AM

**Estimated Time**: 8-10 minutes  
**Required Tier**: Standard (Free) - uses Discord nodes

## 🚀 Step 1: Create Your Account

1. **Visit DeFlow**
   ```
   🌐 https://app.deflow.app
   ```

2. **Sign Up**
   - Click "Get Started" 
   - Choose authentication method:
     - Internet Identity (Recommended)
     - NFID
     - Email + Password

3. **Verify Account**
   - Complete identity verification
   - Accept terms of service
   - Choose Standard tier (free) to start

{% hint style="info" %}
**Internet Identity**: DeFlow uses ICP's decentralized identity system for maximum security and privacy.
{% endhint %}

## 🎨 Step 2: Create Your First Workflow

### Open Workflow Builder

1. **Dashboard Navigation**
   ```
   Dashboard → Create Workflow → Start from Scratch
   ```

2. **Workflow Details**
   - **Name**: "Daily Portfolio Alert"
   - **Description**: "Monitor portfolio and alert on significant gains"
   - **Tags**: #portfolio, #alert, #daily

### Configure Basic Settings

```yaml
Workflow Configuration:
  Name: "Daily Portfolio Alert"
  Description: "Send Discord alert for portfolio gains > 5%"
  Execution: Scheduled
  Status: Draft (for testing)
```

## 🔧 Step 3: Add Workflow Nodes

### Add Schedule Trigger

1. **Find Schedule Node**
   - In Node Palette (left panel)
   - Category: "Triggers"
   - Drag "Schedule Trigger" to canvas

2. **Configure Schedule**
   ```yaml
   Schedule Configuration:
     Type: "Daily"
     Time: "09:00"
     Timezone: "Your Local Timezone"
     Days: "Monday through Friday"
   ```

### Add Portfolio Check Node

1. **Add Bitcoin Portfolio Node**
   - Category: "DeFi"
   - Drag "Bitcoin Portfolio" to canvas
   - Position: Right of Schedule Trigger

2. **Configure Portfolio**
   ```yaml
   Portfolio Configuration:
     Wallet Address: "your-btc-address"
     Check Interval: "On trigger"
     Calculate 24h Change: true
   ```

3. **Connect Nodes**
   - Drag from Schedule Trigger output → Bitcoin Portfolio input
   - Connection appears as blue line

### Add Condition Node

1. **Add Condition Node**
   - Category: "Logic"
   - Drag "Condition" to canvas
   - Position: Right of Portfolio Node

2. **Configure Condition**
   ```yaml
   Condition Logic:
     If: portfolio_change_24h > 5
     Then: Continue to Discord
     Else: End workflow
   ```

### Add Discord Notification

1. **Add Discord Webhook**
   - Category: "Social Media"
   - Drag "Discord Webhook" to canvas
   - Position: Right of Condition Node

2. **Configure Discord**
   ```yaml
   Discord Configuration:
     Webhook URL: "your-discord-webhook-url"
     Message: "🚀 Portfolio Alert! 
              Gain: ${portfolio_change_24h}%
              Value: ${portfolio_value} BTC"
     Color: "Green"
   ```

## 🔗 Step 4: Connect Everything

Your workflow should look like this:

```
[Schedule Trigger] → [Portfolio Check] → [Condition] → [Discord Alert]
                                            ↓
                                       [End (No Alert)]
```

### Connection Guide

1. **Schedule → Portfolio**
   - Drag from "trigger" output to "input" 
   - Ensures portfolio check runs on schedule

2. **Portfolio → Condition**
   - Drag from "portfolio_data" output to "input"
   - Passes portfolio data for evaluation

3. **Condition → Discord** 
   - Drag from "true" output to "input"
   - Only runs if condition is met

4. **Condition → End**
   - Drag from "false" output to workflow end
   - Graceful termination if no alert needed

## ⚙️ Step 5: Configure Discord Webhook

### Create Discord Webhook

1. **Open Discord Server**
   - Go to your Discord server
   - Right-click on channel where you want alerts
   - Select "Edit Channel"

2. **Create Webhook**
   ```
   Integrations → Webhooks → New Webhook
   ```

3. **Configure Webhook**
   - **Name**: "DeFlow Alerts"
   - **Avatar**: Upload DeFlow logo (optional)
   - **Copy Webhook URL**

4. **Add to DeFlow**
   - Paste webhook URL into Discord node
   - Test connection with "Send Test Message"

{% hint style="warning" %}
**Keep Webhook Secure**: Never share your webhook URL publicly as anyone can use it to send messages to your Discord.
{% endhint %}

## 🧪 Step 6: Test Your Workflow

### Manual Test Run

1. **Test Individual Nodes**
   ```
   Click each node → "Test Node" → Review output
   ```

2. **Full Workflow Test**
   ```
   Workflow Builder → "Test Run" → Monitor execution
   ```

3. **Check Results**
   - Verify Discord message received
   - Review execution logs
   - Confirm data flow between nodes

### Debug Common Issues

| Issue | Solution |
|-------|----------|
| **Node Connection Failed** | Check output/input compatibility |
| **Discord Test Failed** | Verify webhook URL is correct |
| **Portfolio Data Empty** | Confirm wallet address is valid |
| **Schedule Not Triggering** | Check timezone settings |

## 📱 Step 7: Deploy & Monitor

### Publish Workflow

1. **Save as Draft**
   ```
   Workflow Builder → Save → "Save as Draft"
   ```

2. **Test Thoroughly**
   - Run multiple test executions
   - Verify all edge cases
   - Check error handling

3. **Publish Live**
   ```
   Workflow Actions → "Publish Workflow"
   Status: Draft → Published
   ```

### Monitor Execution

1. **Execution History**
   ```
   Dashboard → Workflows → "Daily Portfolio Alert" → Execution History
   ```

2. **Performance Metrics**
   - Success rate
   - Average execution time
   - Error frequency
   - Last successful run

## 🎉 Congratulations!

You've successfully created your first DeFlow workflow! Here's what you accomplished:

✅ **Created automated portfolio monitoring**  
✅ **Set up conditional alerting logic**  
✅ **Integrated Discord notifications**  
✅ **Configured scheduled execution**  
✅ **Deployed a live workflow**

## 🚀 Next Steps

### Enhance Your Workflow

1. **Add More Conditions**
   - Alert on losses > 10%
   - Include additional portfolio metrics
   - Add price targets

2. **Multiple Notifications**
   - Email alerts for important changes
   - Telegram notifications for urgency
   - Twitter posts for community

3. **Advanced Features**
   - Portfolio rebalancing triggers
   - Multi-chain monitoring
   - Risk management alerts

### Explore More Templates

{% tabs %}
{% tab title="Social Media" %}
**Cross-Platform Posting**
- Schedule posts across Twitter, Discord, Telegram
- Content calendar automation
- Engagement tracking

**Template**: [Social Media Manager →](../user-guide/workflows/drafts-templates.md)
{% endtab %}

{% tab title="DeFi Strategies" %}
**Yield Farming Optimizer**
- Monitor multiple yield farms
- Automatic position rebalancing
- Gas optimization

**Template**: [DeFi Yield Tracker →](../advanced/defi-strategies.md)
{% endtab %}

{% tab title="Portfolio Management" %}
**Multi-Chain Portfolio**
- Bitcoin, Ethereum, Solana tracking
- Risk assessment alerts
- Performance analytics

**Template**: [Portfolio Tracker →](../user-guide/nodes/defi/portfolio.md)
{% endtab %}
{% endtabs %}

### Upgrade for More Features

As your automation needs grow, consider upgrading:

| Feature Need | Recommended Tier |
|--------------|------------------|
| **Social Media Automation** | Premium ($19/month) |
| **Advanced DeFi Strategies** | Premium ($19/month) |
| **API Development** | Pro ($149/month) |
| **Custom Integrations** | Pro ($149/month) |

[Compare subscription tiers →](../user-guide/subscription/tiers.md)

## 💬 Get Help

- **Community Forum**: Ask questions and share workflows
- **Documentation**: Comprehensive guides for every feature
- **Support Email**: help@deflow.app (Premium+ priority)
- **Discord Community**: Real-time help from users and team

## 🎯 Quick Actions

```yaml
What's Next?
□ Create second workflow
□ Join Discord community  
□ Explore node library
□ Set up additional integrations
□ Share your workflow template
```

Ready to build something more advanced? Check out our [workflow templates](../user-guide/workflows/drafts-templates.md) or dive into [DeFi automation strategies](../advanced/defi-strategies.md)!

---

**Stuck on something?** Don't hesitate to reach out - the DeFlow community is here to help! 🤝