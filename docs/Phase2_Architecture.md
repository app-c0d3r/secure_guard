# Phase 2: MVP Features & Architecture

**Document Version:** 2.0  
**Target Timeline:** Months 4-6  
**Status:** Design Phase

## 🎯 Phase 2 Goals

Transform SecureGuard from a basic agent management system into a complete threat detection and monitoring platform with real-time capabilities.

### Core Objectives
1. **Real-time Threat Detection**: Event-driven security monitoring
2. **Interactive Dashboard**: Web-based agent and threat visualization
3. **Live Communication**: Bidirectional agent command & control
4. **Event Processing**: Scalable security event analysis
5. **Alerting System**: Automated threat notifications

## 🏗 System Architecture Overview

### Enhanced Backend Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   React SPA     │◄──►│   Axum API       │◄──►│   Agents        │
│   Dashboard     │    │   Gateway        │    │   (Windows)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
        │                       │                        │
        │                       ▼                        │
        │              ┌──────────────────┐              │
        │              │  WebSocket Hub   │◄─────────────┘
        │              └──────────────────┘
        │                       │
        ▼                       ▼
┌─────────────────┐    ┌──────────────────┐
│  Notification   │    │ Event Processing │
│  System         │    │ Engine           │
└─────────────────┘    └──────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  Threat Database │
                       │  (PostgreSQL)    │
                       └──────────────────┘
```

## 🛡 Threat Detection Engine

### 1. Event Collection System

#### Security Event Types
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    ProcessStart { pid: u32, executable: String, command_line: String },
    FileAccess { path: String, operation: FileOperation, process: String },
    NetworkConnection { local_port: u16, remote_addr: String, remote_port: u16 },
    RegistryModification { key: String, value: String, operation: RegistryOperation },
    UserLogin { username: String, session_type: String, success: bool },
    SystemChange { component: String, description: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: Uuid,
    pub agent_id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub severity: Severity,
    pub raw_data: serde_json::Value,
}
```

#### Event Sources (Windows Agent)
- **Process Monitoring**: WinAPI Process and Thread events
- **File System Monitoring**: File/Directory access, modifications
- **Network Monitoring**: TCP/UDP connections, DNS queries
- **Registry Monitoring**: Registry key/value changes
- **Authentication Events**: Login attempts, privilege escalations
- **System Events**: Service changes, driver loads, system calls

### 2. Rule-Based Detection Engine

#### Detection Rules
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DetectionRule {
    pub rule_id: Uuid,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuleCondition {
    ProcessName(String),
    ProcessPath(String),
    NetworkConnection { port: Option<u16>, address: Option<String> },
    FileExtension(Vec<String>),
    RegistryKey(String),
    EventFrequency { count: u32, timeframe: Duration },
}

#[derive(Debug, Serialize, Deserialize)]  
pub enum RuleAction {
    Alert,
    QuarantineFile(String),
    KillProcess(u32),
    BlockNetwork,
    LogEvent,
}
```

#### Built-in Security Rules
1. **Malware Detection**
   - Suspicious process execution from temp directories
   - Known malicious file hashes
   - Unsigned executable execution

2. **Network Threats**
   - Connections to known malicious IPs
   - Unusual outbound traffic patterns
   - DNS queries to suspicious domains

3. **System Integrity**
   - Critical system file modifications
   - Registry key changes in security-sensitive areas
   - Service installation/modification

4. **User Behavior**
   - Multiple failed login attempts
   - Privilege escalation attempts
   - Unusual access patterns

### 3. Event Processing Pipeline

#### Real-time Processing Flow
```
Agent Event → WebSocket → Event Ingestion → Rule Engine → Alert Generation → Dashboard Update
     │              │            │              │               │                │
     └──────────────┴────────────┴──────────────┴───────────────┴────────────────┘
                                    ▼
                              Event Storage
                              (PostgreSQL)
```

#### Performance Requirements
- **Event Ingestion**: 10,000+ events/second per agent
- **Rule Processing**: <10ms per event
- **Alert Generation**: <100ms from event to dashboard
- **Storage**: Efficient time-series data storage with partitioning

## 🌐 WebSocket Communication System

### 1. Real-time Agent Communication

#### WebSocket Message Types
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum AgentMessage {
    // Agent to Server
    Heartbeat { status: AgentStatus, metrics: SystemMetrics },
    SecurityEvent(SecurityEvent),
    CommandResponse { command_id: Uuid, result: CommandResult },
    
    // Server to Agent  
    Command { command_id: Uuid, command: AgentCommand },
    ConfigUpdate(AgentConfig),
    RuleUpdate(Vec<DetectionRule>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentCommand {
    ScanFile(String),
    QuarantineFile(String), 
    KillProcess(u32),
    UpdateConfig(AgentConfig),
    CollectSystemInfo,
    RestartAgent,
}
```

#### WebSocket Server Implementation
```rust
// WebSocket handler for agent connections
pub async fn handle_agent_websocket(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    agent_auth: AgentAuth,
) -> Response {
    ws.on_upgrade(move |socket| handle_agent_socket(socket, app_state, agent_auth))
}

// Bidirectional message handling
async fn handle_agent_socket(
    mut socket: WebSocket, 
    state: AppState,
    agent: Agent
) {
    // Handle incoming events and outgoing commands
    // Implement connection pooling and message routing
}
```

### 2. Dashboard Real-time Updates

#### Dashboard WebSocket Messages
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum DashboardMessage {
    AgentStatusUpdate { agent_id: Uuid, status: AgentStatus },
    NewSecurityEvent(SecurityEvent),
    ThreatAlert { alert_id: Uuid, severity: Severity, description: String },
    SystemMetricsUpdate { agent_id: Uuid, metrics: SystemMetrics },
}
```

## 📊 React Dashboard Architecture

### 1. Dashboard Components

#### Main Dashboard Layout
```
┌─────────────────────────────────────────────────────────────────┐
│  Header (User Info, Notifications, Search)                     │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│ │   Agents    │ │   Threats   │ │   Events    │ │   System    │ │
│ │  Overview   │ │   Today     │ │   /Hour     │ │   Health    │ │
│ │    247      │ │     23      │ │    1,432    │ │     98%     │ │
│ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────┐ ┌─────────────────────────────┐ │
│ │     Agent Status Map        │ │     Threat Timeline         │ │
│ │   (Interactive World Map)   │ │   (Real-time Event Stream)  │ │
│ │                             │ │                             │ │
│ └─────────────────────────────┘ └─────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │                    Active Alerts Table                     │ │
│ │  Timestamp | Agent | Threat Type | Severity | Actions      │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

#### Core React Components
```typescript
// Main dashboard container
export const Dashboard: React.FC = () => {
  const { agents, threats, events } = useRealTimeData();
  
  return (
    <DashboardLayout>
      <MetricsOverview agents={agents} threats={threats} events={events} />
      <div className="grid grid-cols-2 gap-6">
        <AgentStatusMap agents={agents} />
        <ThreatTimeline events={events} />
      </div>
      <ActiveAlertsTable threats={threats} />
    </DashboardLayout>
  );
};

// Real-time data hook
export const useRealTimeData = () => {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [threats, setThreats] = useState<Threat[]>([]);
  const { socket } = useWebSocket();
  
  useEffect(() => {
    socket.on('agentUpdate', updateAgent);
    socket.on('newThreat', addThreat);
    return () => socket.off();
  }, []);
  
  return { agents, threats, events };
};
```

### 2. Dashboard Features

#### Agent Management
- **Agent List**: Real-time status, last seen, version info
- **Agent Details**: System information, running processes, network connections
- **Remote Commands**: File scanning, process termination, configuration updates
- **Agent Health**: CPU, memory, disk usage monitoring

#### Threat Monitoring
- **Alert Dashboard**: Active threats, severity levels, auto-refresh
- **Threat Details**: Event timeline, affected systems, remediation steps
- **Threat Hunting**: Search and filter historical events
- **Response Actions**: Quarantine, block, investigate options

#### System Analytics
- **Event Timeline**: Chronological security event visualization
- **Trend Analysis**: Threat patterns over time
- **Agent Performance**: Response times, resource usage
- **Security Metrics**: MTTR, false positive rates, coverage

## 🔧 Technology Stack - Phase 2

### Frontend Technology
```json
{
  "framework": "React 18 + TypeScript",
  "build_tool": "Vite 5",
  "styling": "Tailwind CSS 3 + Headless UI",
  "state_management": "Zustand + React Query",
  "charts": "Recharts + D3.js",
  "websockets": "Socket.io-client",
  "testing": "Vitest + React Testing Library"
}
```

### Backend Enhancements
```toml
# Additional dependencies for Phase 2
tokio-tungstenite = "0.21"  # WebSocket server
serde_yaml = "0.9"          # Rule configuration
regex = "1.0"               # Pattern matching
chrono = { features = ["serde"] }  # Time series data
redis = { features = ["streams"] }  # Event streaming
prometheus = "0.13"         # Metrics collection
```

## 📋 Implementation Roadmap

### Month 4: Threat Detection Foundation
- **Week 1-2**: Event collection system and database schema
- **Week 3-4**: Basic rule engine and detection logic

### Month 5: Real-time Communication
- **Week 1-2**: WebSocket infrastructure and agent communication
- **Week 3-4**: React dashboard foundation and basic UI

### Month 6: Integration & Polish
- **Week 1-2**: Dashboard real-time features and threat visualization
- **Week 3-4**: Testing, performance optimization, beta deployment

## 🎯 Success Metrics

### Performance Targets
- **Event Processing**: <10ms per event
- **Dashboard Responsiveness**: <200ms UI updates
- **Agent Communication**: <100ms command response
- **System Scalability**: Support 1,000+ concurrent agents

### Security Effectiveness
- **Detection Rate**: >95% for known threat patterns
- **False Positives**: <5% of total alerts
- **Response Time**: <60 seconds from detection to alert
- **Coverage**: Monitor 100% of critical system events

---

**Next Steps**: Complete Phase 1 environment setup, then begin threat detection engine implementation.