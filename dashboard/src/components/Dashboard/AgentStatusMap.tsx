import React from 'react';
import {
  VStack,
  HStack,
  Text,
  Badge,
  Box,
  SimpleGrid,
} from '@chakra-ui/react';
import { FiServer, FiWifi, FiWifiOff } from 'react-icons/fi';

// Icon wrapper component to fix TypeScript issues
const IconWrapper = ({ children, ...props }: any) => (
  <Box display="inline-flex" alignItems="center" {...props}>
    {children}
  </Box>
);

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

interface AgentStatusMapProps {
  agents: Agent[];
}

const AgentStatusMap: React.FC<AgentStatusMapProps> = ({ agents }) => {
  const getStatusCounts = () => {
    return {
      online: agents.filter(a => a.status === 'online').length,
      offline: agents.filter(a => a.status === 'offline').length,
      unknown: agents.filter(a => a.status === 'unknown').length,
      error: agents.filter(a => a.status === 'error').length,
      total: agents.length,
    };
  };

  const counts = getStatusCounts();
  const onlinePercentage = counts.total > 0 ? (counts.online / counts.total) * 100 : 0;

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

  return (
    <VStack gap={4} align="stretch">
      {/* Summary Stats */}
      <SimpleGrid columns={2} gap={3}>
        <Box p={3} bg="green.50" border="1px" borderColor="green.200" borderRadius="md">
            <HStack>
              <IconWrapper>
                <Box w={5} h={5} bg="green.500" borderRadius="sm" />
              </IconWrapper>
              <VStack align="start" gap={0}>
                <Text fontSize="lg" fontWeight="bold" color="green.700">
                  {counts.online}
                </Text>
                <Text fontSize="xs" color="green.600">
                  Online
                </Text>
              </VStack>
            </HStack>
        </Box>

        <Box p={3} bg="red.50" border="1px" borderColor="red.200" borderRadius="md">
            <HStack>
              <IconWrapper>
                <Box w={5} h={5} bg="red.500" borderRadius="sm" />
              </IconWrapper>
              <VStack align="start" gap={0}>
                <Text fontSize="lg" fontWeight="bold" color="red.700">
                  {counts.offline + counts.error}
                </Text>
                <Text fontSize="xs" color="red.600">
                  Offline
                </Text>
              </VStack>
            </HStack>
        </Box>
      </SimpleGrid>

      {/* Overall Health */}
      <Box>
        <HStack justify="space-between" mb={2}>
          <Text fontSize="sm" fontWeight="medium">
            Network Health
          </Text>
          <Text fontSize="sm" color="gray.600">
            {onlinePercentage.toFixed(1)}%
          </Text>
        </HStack>
        <Box w="100%" bg="gray.100" borderRadius="md" h="8px">
          <Box
            h="100%"
            bg={onlinePercentage > 80 ? 'green.400' : onlinePercentage > 60 ? 'orange.400' : 'red.400'}
            borderRadius="md"
            width={`${onlinePercentage}%`}
            transition="width 0.3s"
          />
        </Box>
      </Box>

      {/* Recent Agents */}
      <VStack align="stretch" gap={2}>
        <Text fontSize="sm" fontWeight="medium" color="gray.700">
          Recent Agents ({Math.min(agents.length, 5)})
        </Text>
        
        {agents.slice(0, 5).map((agent) => (
          <HStack key={agent.agent_id} justify="space-between" p={2} borderRadius="md" bg="gray.50">
            <HStack gap={3}>
              <IconWrapper>
                <Box w={4} h={4} bg={`${getStatusColor(agent.status)}.500`} borderRadius="sm" />
              </IconWrapper>
              <VStack align="start" gap={0}>
                <Text fontSize="sm" fontWeight="medium" overflow="hidden" textOverflow="ellipsis" whiteSpace="nowrap">
                  {getHostname(agent.os_info)}
                </Text>
                <Text fontSize="xs" color="gray.500" fontFamily="mono">
                  {agent.agent_id.slice(0, 8)}...
                </Text>
              </VStack>
            </HStack>
            
            <VStack align="end" gap={0}>
              <Badge
                size="sm"
                colorScheme={getStatusColor(agent.status)}
                variant="subtle"
              >
                {agent.status}
              </Badge>
              <Text fontSize="xs" color="gray.500">
                {formatLastHeartbeat(agent.last_heartbeat)}
              </Text>
            </VStack>
          </HStack>
        ))}

        {agents.length === 0 && (
          <Box textAlign="center" py={4}>
            <Box w={8} h={8} bg="gray.300" borderRadius="md" mb={2} />
            <Text fontSize="sm" color="gray.500">
              No agents connected
            </Text>
          </Box>
        )}
      </VStack>
    </VStack>
  );
};

export default AgentStatusMap;