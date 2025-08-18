import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  GridItem,
  Text,
  HStack,
  VStack,
  Badge,
  Heading,
} from '@chakra-ui/react';
import {
  FiShield,
  FiServer,
  FiAlertTriangle,
  FiActivity,
  FiCpu,
  FiHardDrive,
} from 'react-icons/fi';
import { useSocket } from '../contexts/SocketContext';
import ThreatActivityChart from '../components/Charts/ThreatActivityChart';
import SystemHealthChart from '../components/Charts/SystemHealthChart';
import RecentAlertsTable from '../components/Dashboard/RecentAlertsTable';
import AgentStatusMap from '../components/Dashboard/AgentStatusMap';

interface DashboardStats {
  totalAgents: number;
  onlineAgents: number;
  offlineAgents: number;
  totalEvents24h: number;
  totalAlerts24h: number;
  criticalAlerts: number;
  systemHealth: {
    cpu: number;
    memory: number;
    disk: number;
  };
  threatTrends: {
    current: number;
    previous: number;
    trend: 'up' | 'down';
  };
}

const Dashboard: React.FC = () => {
  const { agents, alerts, systemMetrics, connected } = useSocket();
  const [stats, setStats] = useState<DashboardStats>({
    totalAgents: 0,
    onlineAgents: 0,
    offlineAgents: 0,
    totalEvents24h: 0,
    totalAlerts24h: 0,
    criticalAlerts: 0,
    systemHealth: { cpu: 0, memory: 0, disk: 0 },
    threatTrends: { current: 0, previous: 0, trend: 'down' },
  });

  const cardBg = 'white';
  const bgColor = 'gray.50';

  useEffect(() => {
    // Update stats based on real-time data
    const onlineAgents = agents.filter(agent => agent.status === 'online').length;
    const offlineAgents = agents.filter(agent => agent.status === 'offline').length;
    const criticalAlerts = alerts.filter(alert => 
      alert.severity === 'critical' && alert.status === 'open'
    ).length;

    const today = new Date();
    const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000);
    
    const alerts24h = alerts.filter(alert => 
      new Date(alert.created_at) >= yesterday
    ).length;

    setStats({
      totalAgents: agents.length,
      onlineAgents,
      offlineAgents,
      totalEvents24h: systemMetrics?.events_24h || 0,
      totalAlerts24h: alerts24h,
      criticalAlerts,
      systemHealth: {
        cpu: systemMetrics?.cpu_usage || 0,
        memory: systemMetrics?.memory_usage || 0,
        disk: systemMetrics?.disk_usage || 0,
      },
      threatTrends: {
        current: alerts24h,
        previous: Math.floor(alerts24h * 0.8), // Mock previous day data
        trend: alerts24h > Math.floor(alerts24h * 0.8) ? 'up' : 'down',
      },
    });
  }, [agents, alerts, systemMetrics]);

  const StatCard = ({ 
    label, 
    value, 
    helpText, 
    icon, 
    colorScheme = 'blue',
    trend,
    trendValue 
  }: {
    label: string;
    value: string | number;
    helpText?: string;
    icon: any;
    colorScheme?: string;
    trend?: 'up' | 'down';
    trendValue?: number;
  }) => (
    <Box bg={cardBg} p={6} shadow="sm" borderRadius="lg" border="1px" borderColor="gray.200">
      <HStack justify="space-between">
        <VStack align="start" gap={1}>
          <HStack>
            <Box w={5} h={5} bg={`${colorScheme}.500`} borderRadius="md" />
            <Text fontSize="sm" color="gray.600" fontWeight="medium">
              {label}
            </Text>
          </HStack>
          <Text fontSize="2xl" fontWeight="bold" color="gray.800">
            {value}
          </Text>
          {(helpText || (trend && trendValue)) && (
            <HStack gap={2}>
              {trend && trendValue && (
                <HStack gap={1}>
                  <Box 
                    w={3} 
                    h={3} 
                    bg={trend === 'up' ? 'red.500' : 'green.500'} 
                    borderRadius="sm"
                    transform={trend === 'up' ? 'rotate(45deg)' : 'rotate(-45deg)'}
                  />
                  <Text fontSize="sm" color={trend === 'up' ? 'red.500' : 'green.500'}>
                    {trendValue}%
                  </Text>
                </HStack>
              )}
              {helpText && (
                <Text fontSize="sm" color="gray.500">
                  {helpText}
                </Text>
              )}
            </HStack>
          )}
        </VStack>
      </HStack>
    </Box>
  );

  return (
    <Box bg={bgColor} minH="100vh">
      <VStack gap={6} align="stretch">
        {/* Header */}
        <HStack justify="space-between">
          <Heading size="lg" color="gray.800">
            Security Overview
          </Heading>
          <HStack>
            <Badge
              colorScheme={connected ? 'green' : 'red'}
              px={3}
              py={1}
              borderRadius="full"
            >
              {connected ? 'Live Monitoring Active' : 'Offline Mode'}
            </Badge>
          </HStack>
        </HStack>

        {/* Key Metrics */}
        <Grid templateColumns="repeat(auto-fit, minmax(250px, 1fr))" gap={4}>
          <StatCard
            label="Total Agents"
            value={stats.totalAgents}
            helpText={`${stats.onlineAgents} online, ${stats.offlineAgents} offline`}
            icon={FiServer}
            colorScheme="blue"
          />
          <StatCard
            label="Security Events (24h)"
            value={stats.totalEvents24h.toLocaleString()}
            helpText="Events processed"
            icon={FiActivity}
            colorScheme="purple"
          />
          <StatCard
            label="Active Threats"
            value={stats.criticalAlerts}
            helpText="Critical alerts requiring attention"
            icon={FiAlertTriangle}
            colorScheme="red"
          />
          <StatCard
            label="System Health"
            value={`${Math.round((100 - stats.systemHealth.cpu - stats.systemHealth.memory) / 2)}%`}
            helpText="Overall system performance"
            icon={FiShield}
            colorScheme="green"
          />
        </Grid>

        {/* Charts and Detailed Views */}
        <Grid templateColumns="2fr 1fr" gap={6}>
          {/* Threat Activity Chart */}
          <GridItem>
            <Box bg={cardBg} p={6} shadow="sm" borderRadius="lg" border="1px" borderColor="gray.200">
              <VStack gap={4} align="stretch">
                <Heading size="md">Threat Activity (Last 24 Hours)</Heading>
                <ThreatActivityChart alerts={alerts} />
              </VStack>
            </Box>
          </GridItem>

          {/* System Health */}
          <GridItem>
            <Box bg={cardBg} p={6} shadow="sm" borderRadius="lg" border="1px" borderColor="gray.200">
              <VStack gap={4} align="stretch">
                <Heading size="md">System Resources</Heading>
                
                <Box>
                  <HStack justify="space-between" mb={2}>
                    <HStack>
                      <Box w={4} h={4} bg="blue.500" borderRadius="sm" />
                      <Text fontSize="sm" fontWeight="medium">CPU Usage</Text>
                    </HStack>
                    <Text fontSize="sm" color="gray.600">
                      {stats.systemHealth.cpu.toFixed(1)}%
                    </Text>
                  </HStack>
                  <Box w="100%" bg="gray.200" borderRadius="md" h="8px">
                    <Box
                      h="100%"
                      bg={stats.systemHealth.cpu > 80 ? 'red.400' : stats.systemHealth.cpu > 60 ? 'orange.400' : 'green.400'}
                      borderRadius="md"
                      width={`${stats.systemHealth.cpu}%`}
                      transition="width 0.3s"
                    />
                  </Box>
                </Box>

                <Box>
                  <HStack justify="space-between" mb={2}>
                    <HStack>
                      <Box w={4} h={4} bg="purple.500" borderRadius="sm" />
                      <Text fontSize="sm" fontWeight="medium">Memory Usage</Text>
                    </HStack>
                    <Text fontSize="sm" color="gray.600">
                      {stats.systemHealth.memory.toFixed(1)}%
                    </Text>
                  </HStack>
                  <Box w="100%" bg="gray.200" borderRadius="md" h="8px">
                    <Box
                      h="100%"
                      bg={stats.systemHealth.memory > 80 ? 'red.400' : stats.systemHealth.memory > 60 ? 'orange.400' : 'green.400'}
                      borderRadius="md"
                      width={`${stats.systemHealth.memory}%`}
                      transition="width 0.3s"
                    />
                  </Box>
                </Box>

                <Box>
                  <HStack justify="space-between" mb={2}>
                    <HStack>
                      <Box w={4} h={4} bg="green.500" borderRadius="sm" />
                      <Text fontSize="sm" fontWeight="medium">Disk Usage</Text>
                    </HStack>
                    <Text fontSize="sm" color="gray.600">
                      {stats.systemHealth.disk.toFixed(1)}%
                    </Text>
                  </HStack>
                  <Box w="100%" bg="gray.200" borderRadius="md" h="8px">
                    <Box
                      h="100%"
                      bg={stats.systemHealth.disk > 80 ? 'red.400' : stats.systemHealth.disk > 60 ? 'orange.400' : 'green.400'}
                      borderRadius="md"
                      width={`${stats.systemHealth.disk}%`}
                      transition="width 0.3s"
                    />
                  </Box>
                </Box>
              </VStack>
            </Box>
          </GridItem>
        </Grid>

        {/* Recent Alerts and Agent Status */}
        <Grid templateColumns="3fr 2fr" gap={6}>
          <GridItem>
            <Box bg={cardBg} p={6} shadow="sm" borderRadius="lg" border="1px" borderColor="gray.200">
              <VStack gap={4} align="stretch">
                <Heading size="md">Recent Threat Alerts</Heading>
                <RecentAlertsTable alerts={alerts.slice(0, 10)} />
              </VStack>
            </Box>
          </GridItem>

          <GridItem>
            <Box bg={cardBg} p={6} shadow="sm" borderRadius="lg" border="1px" borderColor="gray.200">
              <VStack gap={4} align="stretch">
                <Heading size="md">Agent Status Overview</Heading>
                <AgentStatusMap agents={agents} />
              </VStack>
            </Box>
          </GridItem>
        </Grid>
      </VStack>
    </Box>
  );
};

export default Dashboard;