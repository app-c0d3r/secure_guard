import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import io from 'socket.io-client';
import { useAuth } from './AuthContext';

type SocketType = ReturnType<typeof io>;

interface SocketContextType {
  socket: SocketType | null;
  connected: boolean;
  agents: Agent[];
  alerts: ThreatAlert[];
  systemMetrics: SystemMetrics | null;
}

interface Agent {
  agent_id: string;
  tenant_id: string;
  hardware_fingerprint: string;
  os_info: any;
  status: 'online' | 'offline' | 'unknown' | 'error';
  last_heartbeat: string | null;
  version: string;
  created_at: string;
}

interface ThreatAlert {
  alert_id: string;
  event_id: string;
  rule_id: string | null;
  agent_id: string;
  alert_type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string | null;
  status: 'open' | 'investigating' | 'resolved' | 'false_positive';
  assigned_to: string | null;
  resolved_at: string | null;
  created_at: string;
  updated_at: string;
}

interface SystemMetrics {
  total_agents: number;
  online_agents: number;
  offline_agents: number;
  events_24h: number;
  alerts_24h: number;
  critical_alerts: number;
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
}

const SocketContext = createContext<SocketContextType | undefined>(undefined);

export const useSocket = () => {
  const context = useContext(SocketContext);
  if (context === undefined) {
    throw new Error('useSocket must be used within a SocketProvider');
  }
  return context;
};

interface SocketProviderProps {
  children: ReactNode;
}

export const SocketProvider: React.FC<SocketProviderProps> = ({ children }) => {
  const { token, isAuthenticated } = useAuth();
  const [socket, setSocket] = useState<SocketType | null>(null);
  const [connected, setConnected] = useState(false);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [alerts, setAlerts] = useState<ThreatAlert[]>([]);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null);

  useEffect(() => {
    if (isAuthenticated && token) {
      const newSocket = io('ws://localhost:3000/dashboard', {
        auth: {
          token,
        },
        transports: ['websocket']
      });

      newSocket.on('connect', () => {
        console.log('Connected to SecureGuard WebSocket');
        setConnected(true);
      });

      newSocket.on('disconnect', () => {
        console.log('Disconnected from SecureGuard WebSocket');
        setConnected(false);
      });

      // Real-time event handlers
      newSocket.on('agent_status_update', (data: any) => {
        setAgents(prev => prev.map(agent => 
          agent.agent_id === data.agent_id 
            ? { ...agent, status: data.status, last_heartbeat: data.last_seen }
            : agent
        ));
      });

      newSocket.on('new_threat_alert', (data: any) => {
        setAlerts(prev => [data.alert, ...prev.slice(0, 99)]); // Keep last 100 alerts
      });

      newSocket.on('system_metrics_update', (data: SystemMetrics) => {
        setSystemMetrics(data);
      });

      newSocket.on('batch_processing_summary', (data: any) => {
        console.log('Batch processing summary:', data);
      });

      setSocket(newSocket);

      return () => {
        newSocket.close();
        setSocket(null);
        setConnected(false);
      };
    }
  }, [isAuthenticated, token]);

  const value = {
    socket,
    connected,
    agents,
    alerts,
    systemMetrics,
  };

  return <SocketContext.Provider value={value}>{children}</SocketContext.Provider>;
};