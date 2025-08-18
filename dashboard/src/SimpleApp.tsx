import React, { useState } from 'react';
import { Box, Heading, Text, Button, Input, VStack, HStack, Badge, SimpleGrid } from '@chakra-ui/react';
import Admin from './pages/Admin';

const SimpleApp: React.FC = () => {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [currentView, setCurrentView] = useState('dashboard'); // 'dashboard' or 'admin'
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');

  const handleLogin = () => {
    if (username === 'admin' && password === 'admin123') {
      setIsLoggedIn(true);
    } else {
      alert('Invalid credentials. Use admin/admin123');
    }
  };

  if (!isLoggedIn) {
    return (
      <Box
        minH="100vh"
        display="flex"
        alignItems="center"
        justifyContent="center"
        bg="gray.50"
      >
        <Box
          maxW="md"
          mx="auto"
          bg="white"
          p={8}
          borderRadius="xl"
          boxShadow="lg"
        >
          <VStack gap={6}>
            <Box textAlign="center">
              <Heading size="lg" color="blue.500" mb={2}>
                🛡️ SecureGuard
              </Heading>
              <Text color="gray.600" fontSize="sm">
                Cybersecurity Management Platform
              </Text>
            </Box>

            <VStack gap={4} w="full">
              <Box w="full">
                <Text mb={2} fontSize="sm" fontWeight="medium">
                  Username
                </Text>
                <Input
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="Enter username"
                />
              </Box>

              <Box w="full">
                <Text mb={2} fontSize="sm" fontWeight="medium">
                  Password
                </Text>
                <Input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter password"
                />
              </Box>

              <Button
                w="full"
                colorScheme="blue"
                onClick={handleLogin}
              >
                Sign In
              </Button>
            </VStack>

            <Box w="full" p={4} bg="blue.50" borderRadius="md">
              <Text fontSize="sm" fontWeight="semibold" color="blue.700" mb={2}>
                Demo Credentials
              </Text>
              <Text fontSize="xs" color="blue.600">
                Username: <strong>admin</strong>
              </Text>
              <Text fontSize="xs" color="blue.600">
                Password: <strong>admin123</strong>
              </Text>
            </Box>
          </VStack>
        </Box>
      </Box>
    );
  }

  // Mock data
  const agentStats = {
    total: 47,
    online: 43,
    offline: 4,
    critical_alerts: 3,
    events_24h: 15647,
  };

  const recentAlerts = [
    {
      id: 1,
      title: "Suspicious PowerShell Activity",
      severity: "critical",
      agent: "WIN-DESKTOP-001",
      time: "2 minutes ago",
    },
    {
      id: 2,
      title: "Failed Login Attempt",
      severity: "medium",
      agent: "WIN-DESKTOP-012",
      time: "5 minutes ago",
    },
    {
      id: 3,
      title: "Unusual Network Traffic",
      severity: "high",
      agent: "WIN-DESKTOP-007",
      time: "12 minutes ago",
    },
  ];

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'red';
      case 'high': return 'orange';
      case 'medium': return 'yellow';
      default: return 'green';
    }
  };

  // Show admin panel if admin view is selected
  if (currentView === 'admin') {
    return <Admin onBackToDashboard={() => setCurrentView('dashboard')} />;
  }

  return (
    <Box minH="100vh" bg="gray.50">
      {/* Header */}
      <Box bg="white" borderBottom="1px" borderColor="gray.200" px={6} py={4}>
        <HStack justify="space-between">
          <HStack>
            <Heading size="lg" color="blue.500">
              🛡️ SecureGuard Dashboard
            </Heading>
            <Badge colorScheme="green" px={3} py={1}>
              Live Monitoring
            </Badge>
          </HStack>
          <HStack>
            <Text fontSize="sm">Welcome, admin</Text>
            <Button 
              size="sm" 
              colorScheme="blue" 
              variant="outline"
              onClick={() => setCurrentView('admin')}
              mr={2}
            >
              Admin Panel
            </Button>
            <Button size="sm" variant="outline" onClick={() => setIsLoggedIn(false)}>
              Logout
            </Button>
          </HStack>
        </HStack>
      </Box>

      {/* Main Content */}
      <Box p={6}>
        <VStack gap={6} align="stretch">
          {/* Stats Grid */}
          <SimpleGrid columns={{ base: 2, md: 4 }} gap={4}>
            <Box bg="white" p={6} borderRadius="lg" boxShadow="sm">
              <Text fontSize="sm" color="gray.600" mb={2}>
                Total Agents
              </Text>
              <Text fontSize="3xl" fontWeight="bold" color="blue.600">
                {agentStats.total}
              </Text>
              <Text fontSize="xs" color="gray.500">
                {agentStats.online} online, {agentStats.offline} offline
              </Text>
            </Box>

            <Box bg="white" p={6} borderRadius="lg" boxShadow="sm">
              <Text fontSize="sm" color="gray.600" mb={2}>
                Security Events (24h)
              </Text>
              <Text fontSize="3xl" fontWeight="bold" color="purple.600">
                {agentStats.events_24h.toLocaleString()}
              </Text>
              <Text fontSize="xs" color="gray.500">
                Events processed
              </Text>
            </Box>

            <Box bg="white" p={6} borderRadius="lg" boxShadow="sm">
              <Text fontSize="sm" color="gray.600" mb={2}>
                Critical Alerts
              </Text>
              <Text fontSize="3xl" fontWeight="bold" color="red.600">
                {agentStats.critical_alerts}
              </Text>
              <Text fontSize="xs" color="gray.500">
                Require immediate attention
              </Text>
            </Box>

            <Box bg="white" p={6} borderRadius="lg" boxShadow="sm">
              <Text fontSize="sm" color="gray.600" mb={2}>
                System Health
              </Text>
              <Text fontSize="3xl" fontWeight="bold" color="green.600">
                98%
              </Text>
              <Text fontSize="xs" color="gray.500">
                All systems operational
              </Text>
            </Box>
          </SimpleGrid>

          {/* Recent Alerts */}
          <Box bg="white" p={6} borderRadius="lg" boxShadow="sm">
            <Heading size="md" mb={4}>
              Recent Threat Alerts
            </Heading>
            <VStack gap={3} align="stretch">
              {recentAlerts.map((alert) => (
                <Box
                  key={alert.id}
                  p={4}
                  borderRadius="md"
                  border="1px"
                  borderColor="gray.200"
                  _hover={{ bg: 'gray.50' }}
                >
                  <HStack justify="space-between">
                    <VStack align="start" gap={1}>
                      <Text fontWeight="medium" fontSize="sm">
                        {alert.title}
                      </Text>
                      <HStack>
                        <Text fontSize="xs" color="gray.500">
                          Agent: {alert.agent}
                        </Text>
                        <Text fontSize="xs" color="gray.400">
                          • {alert.time}
                        </Text>
                      </HStack>
                    </VStack>
                    <Badge
                      colorScheme={getSeverityColor(alert.severity)}
                      textTransform="uppercase"
                      fontSize="10px"
                    >
                      {alert.severity}
                    </Badge>
                  </HStack>
                </Box>
              ))}
            </VStack>
          </Box>

          {/* Agent Status */}
          <SimpleGrid columns={{ base: 1, lg: 2 }} gap={6}>
            <Box bg="white" p={6} borderRadius="lg" boxShadow="sm">
              <Heading size="md" mb={4}>
                Agent Network Status
              </Heading>
              <VStack gap={3} align="stretch">
                <HStack justify="space-between">
                  <HStack>
                    <Box w={3} h={3} borderRadius="full" bg="green.400" />
                    <Text fontSize="sm">Online Agents</Text>
                  </HStack>
                  <Text fontSize="sm" fontWeight="bold" color="green.600">
                    {agentStats.online}
                  </Text>
                </HStack>
                <HStack justify="space-between">
                  <HStack>
                    <Box w={3} h={3} borderRadius="full" bg="red.400" />
                    <Text fontSize="sm">Offline Agents</Text>
                  </HStack>
                  <Text fontSize="sm" fontWeight="bold" color="red.600">
                    {agentStats.offline}
                  </Text>
                </HStack>
                <Box w="full" h={3} bg="gray.200" borderRadius="full" overflow="hidden">
                  <Box
                    w={`${(agentStats.online / agentStats.total) * 100}%`}
                    h="full"
                    bg="green.400"
                  />
                </Box>
                <Text fontSize="xs" color="gray.500" textAlign="center">
                  {((agentStats.online / agentStats.total) * 100).toFixed(1)}% of agents online
                </Text>
              </VStack>
            </Box>

            <Box bg="white" p={6} borderRadius="lg" boxShadow="sm">
              <Heading size="md" mb={4}>
                Platform Information
              </Heading>
              <VStack gap={3} align="stretch">
                <HStack justify="space-between">
                  <Text fontSize="sm" color="gray.600">
                    Platform Version
                  </Text>
                  <Text fontSize="sm" fontWeight="medium">
                    SecureGuard v2.5.0
                  </Text>
                </HStack>
                <HStack justify="space-between">
                  <Text fontSize="sm" color="gray.600">
                    Database Status
                  </Text>
                  <Badge colorScheme="green" size="sm">
                    Connected
                  </Badge>
                </HStack>
                <HStack justify="space-between">
                  <Text fontSize="sm" color="gray.600">
                    Backend API
                  </Text>
                  <Badge colorScheme="green" size="sm">
                    Operational
                  </Badge>
                </HStack>
                <HStack justify="space-between">
                  <Text fontSize="sm" color="gray.600">
                    Last Updated
                  </Text>
                  <Text fontSize="sm" color="gray.500">
                    {new Date().toLocaleTimeString()}
                  </Text>
                </HStack>
              </VStack>
            </Box>
          </SimpleGrid>

          {/* Footer */}
          <Box textAlign="center" py={6}>
            <Text fontSize="sm" color="gray.500">
              🛡️ SecureGuard Cybersecurity Platform - Advanced Threat Detection & Response
            </Text>
            <Text fontSize="xs" color="gray.400" mt={1}>
              Built with Rust + React • Real-time Monitoring • AI-Powered Detection
            </Text>
          </Box>
        </VStack>
      </Box>
    </Box>
  );
};

export default SimpleApp;