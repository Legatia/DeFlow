import React, { useState, useEffect } from 'react';
import AdminAuth from './components/AdminAuth';
import AdminDashboard from './pages/AdminDashboard';
import SecurityGuard from './components/SecurityGuard';
import SetupMode from './components/SetupMode';
import { AdminAuthService } from './services/adminAuthService';

interface AdminSession {
  principal: string;
  isOwner: boolean;
  sessionStart: number;
}

const App: React.FC = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [adminSession, setAdminSession] = useState<AdminSession | null>(null);
  const [isSetupMode, setIsSetupMode] = useState(false);

  useEffect(() => {
    checkAuthentication();
  }, []);

  const checkAuthentication = async () => {
    try {
      setIsLoading(true);
      const session = await AdminAuthService.getCurrentSession();
      if (session) {
        setAdminSession(session);
        setIsAuthenticated(true);
        setIsSetupMode(!session.isOwner); // Setup mode if not owner yet
      }
    } catch (error) {
      console.error('Authentication check failed:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogin = async (principal: string) => {
    try {
      const session = await AdminAuthService.createSession(principal);
      setAdminSession(session);
      setIsAuthenticated(true);
      setIsSetupMode(!session.isOwner); // Setup mode if not owner yet
    } catch (error) {
      console.error('Login failed:', error);
      throw error;
    }
  };

  const handleLogout = async () => {
    try {
      await AdminAuthService.logout();
      setAdminSession(null);
      setIsAuthenticated(false);
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
          <p className="text-gray-300 mt-4">Loading DeFlow Admin Dashboard...</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <AdminAuth onLogin={handleLogin} />;
  }

  // Setup mode: Show setup interface for initial owner configuration
  if (isSetupMode && adminSession) {
    return (
      <SetupMode
        userPrincipal={adminSession.principal}
        onSetupComplete={() => {
          // After setup, the user needs to refresh or redeploy
          window.location.reload();
        }}
      />
    );
  }

  return (
    <SecurityGuard>
      <AdminDashboard 
        adminSession={adminSession!} 
        onLogout={handleLogout}
      />
    </SecurityGuard>
  );
};

export default App;