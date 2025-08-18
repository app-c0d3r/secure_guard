import React from 'react';
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Badge,
  Button,
  SimpleGrid,
} from '@chakra-ui/react';
import { FiServer, FiWifi, FiWifiOff, FiSettings, FiRefreshCw } from 'react-icons/fi';
import { useSocket } from '../contexts/SocketContext';

const Agents: React.FC = () => {
  const { agents, connected } = useSocket();

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online':
        return 'green';
      case 'offline':
        return 'red';
      case 'error':
        return 'red';
      case 'unknown':
        return 'gray';
      default:
        return 'blue';
    }
  };

  const getHostname = (osInfo: any) => {
    try {
      if (typeof osInfo === 'string') {
        const parsed = JSON.parse(osInfo);
        return parsed.hostname || 'Unknown';
      }
      return osInfo?.hostname || 'Unknown';
    } catch {
      return 'Unknown';
    }
  };

  const getOS = (osInfo: any) => {
    try {
      if (typeof osInfo === 'string') {
        const parsed = JSON.parse(osInfo);
        return parsed.os || 'Unknown OS';
      }
      return osInfo?.os || 'Unknown OS';
    } catch {
      return 'Unknown OS';
    }
  };

  const formatLastHeartbeat = (heartbeat: string | null) => {
    if (!heartbeat) return 'Never';
    const date = new Date(heartbeat);
    const now = new Date();
    const diffInMinutes = Math.floor((now.getTime() - date.getTime()) / (1000 * 60));
    
    if (diffInMinutes < 1) return 'Just now';
    if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
    if (diffInMinutes < 1440) return `${Math.floor(diffInMinutes / 60)}h ago`;
    return `${Math.floor(diffInMinutes / 1440)}d ago`;
  };

  const statusCounts = {
    online: agents.filter(a => a.status === 'online').length,
    offline: agents.filter(a => a.status === 'offline').length,
    error: agents.filter(a => a.status === 'error').length,
    unknown: agents.filter(a => a.status === 'unknown').length,
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Heading size="lg">Agent Management</Heading>
        <HStack>
          <Badge colorScheme={connected ? 'green' : 'red'} px={3} py={1}>
            {connected ? 'Live' : 'Offline'}
          </Badge>
          <Button size="sm" variant="outline">
            Refresh
          </Button>
        </HStack>
      </HStack>

      {/* Status Overview */}
      <SimpleGrid columns={{ base: 2, md: 4 }} gap={4}>
        <Box p={4} bg="white" border="1px" borderColor="gray.200" borderRadius="md">
          <HStack>
            <Box w={6} h={6} bg="green.500" borderRadius="md" />
            <VStack align="start" gap={0}>
              <Text fontSize="2xl" fontWeight="bold" color="green.600">
                {statusCounts.online}
              </Text>
              <Text fontSize="sm" color="gray.600">Online</Text>
            </VStack>
          </HStack>
        </Box>

        <Box p={4} bg="white" border="1px" borderColor="gray.200" borderRadius="md">
          <HStack>
            <Box w={6} h={6} bg="red.500" borderRadius="md" />
            <VStack align="start" gap={0}>
              <Text fontSize="2xl" fontWeight="bold" color="red.600">
                {statusCounts.offline}
              </Text>
              <Text fontSize="sm" color="gray.600">Offline</Text>
            </VStack>
          </HStack>
        </Box>

        <Box p={4} bg="white" border="1px" borderColor="gray.200" borderRadius="md">
          <HStack>
            <Box w={6} h={6} bg="orange.500" borderRadius="md" />
            <VStack align="start" gap={0}>
              <Text fontSize="2xl" fontWeight="bold" color="orange.600">
                {statusCounts.error}
              </Text>
              <Text fontSize="sm" color="gray.600">Error</Text>
            </VStack>
          </HStack>
        </Box>

        <Box p={4} bg="white" border="1px" borderColor="gray.200" borderRadius="md">
          <HStack>
            <Box w={6} h={6} bg="gray.500" borderRadius="md" />
            <VStack align="start" gap={0}>
              <Text fontSize="2xl" fontWeight="bold" color="gray.600">
                {agents.length}
              </Text>
              <Text fontSize="sm" color="gray.600">Total</Text>
            </VStack>
          </HStack>
        </Box>
      </SimpleGrid>

      {/* Agents Table */}
      <Box p={6} bg="white" border="1px" borderColor="gray.200" borderRadius="md">
        <VStack align="stretch" gap={4}>
          <HStack justify="space-between">
            <Heading size="md">Connected Agents</Heading>
            <Text fontSize="sm" color="gray.500">
              {agents.length} agents registered
            </Text>
          </HStack>

          <Box overflowX="auto">
            {/* Custom Table Header */}
            <Box bg="gray.50" p={3} borderRadius="md" mb={2}>
              <HStack>
                <Text fontWeight="medium" fontSize="sm" flex="2">Agent</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Status</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1.5">Operating System</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Version</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Last Heartbeat</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Actions</Text>
              </HStack>
            </Box>

            {/* Custom Table Body */}
            <VStack gap={0}>
              {agents.map((agent, index) => (
                <Box key={agent.agent_id} w="100%">
                  <HStack p={3} _hover={{ bg: 'gray.50' }}>
                    <Box flex="2">
                      <HStack>
                        <Box w={8} h={8} bg={`${getStatusColor(agent.status)}.500`} borderRadius="full" />
                        <VStack align="start" gap={0}>
                          <Text fontWeight="medium">
                            {getHostname(agent.os_info)}
                          </Text>
                          <Text fontSize="xs" color="gray.500" fontFamily="mono">
                            {agent.agent_id.slice(0, 12)}...
                          </Text>
                        </VStack>
                      </HStack>
                    </Box>
                    <Box flex="1">
                      <Badge
                        colorScheme={getStatusColor(agent.status)}
                        variant="subtle"
                      >
                        {agent.status}
                      </Badge>
                    </Box>
                    <Box flex="1.5">
                      <Text fontSize="sm">{getOS(agent.os_info)}</Text>
                    </Box>
                    <Box flex="1">
                      <Text fontSize="sm" fontFamily="mono">
                        v{agent.version}
                      </Text>
                    </Box>
                    <Box flex="1">
                      <Text fontSize="sm" color="gray.600">
                        {formatLastHeartbeat(agent.last_heartbeat)}
                      </Text>
                    </Box>
                    <Box flex="1">
                      <Button size="sm" variant="outline">
                        Manage
                      </Button>
                    </Box>
                  </HStack>
                  {index < agents.length - 1 && <Box h="1px" bg="gray.100" />}
                </Box>
              ))}
            </VStack>

            {agents.length === 0 && (
              <Box textAlign="center" py={10}>
                <Box w={12} h={12} bg="gray.300" borderRadius="md" mx="auto" mb={4} />
                <Text color="gray.500" fontSize="lg">
                  No agents connected
                </Text>
                <Text color="gray.400" fontSize="sm">
                  Agents will appear here when they connect to the platform
                </Text>
              </Box>
            )}
          </Box>
        </VStack>
      </Box>
    </VStack>
  );
};

export default Agents;