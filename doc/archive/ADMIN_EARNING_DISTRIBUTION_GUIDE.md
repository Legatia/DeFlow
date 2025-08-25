# Admin Dashboard - Earning Distribution Testing Guide

## How to Test the Earning Distribution Feature

### Step 1: Access Admin Dashboard
1. Open: http://uxrrr-q7777-77774-qaaaq-cai.localhost:8080/
2. You'll see the login screen

### Step 2: Login as Owner
1. Use this owner principal to login: 
   ```
   npfah-vwoik-mflcp-omami-34o5z-blm5n-zsvsh-hxfog-lbis4-4ghxs-qqe
   ```
2. After login, you should see the admin dashboard with tabs: Treasury Management, Pool Management, System Health, **Team Management**

### Step 3: Add Team Members
1. Click on the **"Team Management"** tab (👥 icon)
2. In the "Add Team Member" section:
   - Enter a test principal (e.g., `rdmx6-jaaaa-aaaah-qcaiq-cai`)
   - Select role: Member or Admin  
   - Click "Add Team Member (Auto-Approve)" (since you're the owner)

3. Add a few more team members:
   - `rrkah-fqaaa-aaaah-qcaiq-cai`
   - `be2us-64aaa-aaaah-qc6hq-cai`

### Step 4: View Earning Distribution Features
Once team members are added, you'll see these NEW sections appear:

#### A. Earning Distribution Summary
- Shows total percentage allocated to team members
- Shows your remaining owner share 
- Warns if total allocation exceeds 100%

#### B. Individual Team Member Cards
Each team member now shows:
- **"Earning Share"** section at the bottom of their card
- Current earning percentage (defaults to 0%)
- **"Edit"** button to modify their percentage (owner only)

### Step 5: Customize Earning Percentages
1. Click **"Edit"** next to any team member's earning share
2. Enter a percentage (0-100, can use decimals like 15.5)
3. Click **"Save"** to apply
4. The summary will automatically update to show:
   - Total allocated to team: sum of all percentages
   - Your owner share: 100% - total team allocation

### Step 6: View Treasury Earnings Tab
1. Go to **"Treasury Management"** tab (💰 icon) 
2. Click **"Team Earnings"** sub-tab (💎 icon)
3. You'll see:
   - Monthly revenue breakdown
   - Individual earnings based on percentages
   - Planned withdrawal system features

## Key Features Implemented:

### 1. **Customizable Percentages**
- Owner can set any percentage (0-100%) for each team member
- Supports decimal precision (e.g., 15.5%)
- Owner automatically gets remainder (100% - total team allocation)

### 2. **Real-time Validation**
- Warning if total allocation exceeds 100%
- Prevents negative or invalid percentages
- Visual feedback for all changes

### 3. **Visual Dashboard**
- Color-coded earning distribution summary
- Individual team member earning cards
- Progress indicators and status messages

### 4. **Treasury Integration** 
- Demo earnings display based on percentages
- Planned withdrawal system preview
- Revenue breakdown visualization

## Testing Scenarios:

1. **No Team Members**: Earning features are hidden
2. **With Team Members**: All earning features become visible
3. **Over-allocation**: System warns when total > 100%
4. **Edit Mode**: Real-time percentage editing interface
5. **Owner View**: Full control over all percentages

The earning distribution system is now fully functional with a complete UI for managing custom percentage allocations!