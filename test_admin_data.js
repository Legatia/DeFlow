// Test script to populate admin data for testing
// Run this in browser console when logged in as owner

// Add some test team members
const testMembers = [
  {
    principal: "rdmx6-jaaaa-aaaah-qcaiq-cai",
    addedBy: "owner-principal",
    addedAt: Date.now() - 86400000, // 1 day ago
    role: "admin",
    status: "active",
    earningPercentage: 15
  },
  {
    principal: "rrkah-fqaaa-aaaah-qcaiq-cai", 
    addedBy: "owner-principal",
    addedAt: Date.now() - 172800000, // 2 days ago
    role: "member",
    status: "active",
    earningPercentage: 12
  },
  {
    principal: "be2us-64aaa-aaaah-qc6hq-cai",
    addedBy: "owner-principal", 
    addedAt: Date.now() - 259200000, // 3 days ago
    role: "member",
    status: "active",
    earningPercentage: 8
  }
];

// Store test data
localStorage.setItem('deflow_team_members', JSON.stringify(testMembers));

console.log('Test team members added! Refresh the page to see the earning distribution UI.');
console.log('Team members:', testMembers);