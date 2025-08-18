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
import { FiAlertTriangle, FiShield, FiActivity, FiRefreshCw } from 'react-icons/fi';
import { useSocket } from '../contexts/SocketContext';

const Threats: React.FC = () => {
  const { alerts, connected } = useSocket();

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'red';
      case 'high':
        return 'orange';
      case 'medium':
        return 'yellow';
      case 'low':
        return 'green';
      default:
        return 'gray';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'open':
        return 'red';
      case 'investigating':
        return 'orange';
      case 'resolved':
        return 'green';
      case 'false_positive':
        return 'gray';
      default:
        return 'blue';
    }
  };

  const formatTimeAgo = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffInMinutes = Math.floor((now.getTime() - date.getTime()) / (1000 * 60));
    
    if (diffInMinutes < 1) return 'Just now';
    if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
    if (diffInMinutes < 1440) return `${Math.floor(diffInMinutes / 60)}h ago`;
    return `${Math.floor(diffInMinutes / 1440)}d ago`;
  };

  const severityCounts = {
    critical: alerts.filter(a => a.severity === 'critical' && a.status === 'open').length,
    high: alerts.filter(a => a.severity === 'high' && a.status === 'open').length,
    medium: alerts.filter(a => a.severity === 'medium' && a.status === 'open').length,
    low: alerts.filter(a => a.severity === 'low' && a.status === 'open').length,
  };

  const statusCounts = {
    open: alerts.filter(a => a.status === 'open').length,
    investigating: alerts.filter(a => a.status === 'investigating').length,
    resolved: alerts.filter(a => a.status === 'resolved').length,
    false_positive: alerts.filter(a => a.status === 'false_positive').length,
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Heading size="lg">Threat Management</Heading>
        <HStack>
          <Badge colorScheme={connected ? 'green' : 'red'} px={3} py={1}>
            {connected ? 'Live Monitoring' : 'Offline'}
          </Badge>
          <Button size="sm" variant="outline">
            Refresh
          </Button>
        </HStack>
      </HStack>

      {/* Severity Overview */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
        <VStack align="stretch" gap={4}>
          <Heading size="md">Threat Severity Distribution</Heading>
          <SimpleGrid columns={{ base: 2, md: 4 }} gap={4}>
            <Box textAlign="center" p={4} borderRadius="md" bg="red.50">
              <Text fontSize="3xl" fontWeight="bold" color="red.600">
                {severityCounts.critical}
              </Text>
              <Text fontSize="sm" color="red.700" fontWeight="medium">
                Critical
              </Text>
            </Box>
            <Box textAlign="center" p={4} borderRadius="md" bg="orange.50">
              <Text fontSize="3xl" fontWeight="bold" color="orange.600">
                {severityCounts.high}
              </Text>
              <Text fontSize="sm" color="orange.700" fontWeight="medium">
                High
              </Text>
            </Box>
            <Box textAlign="center" p={4} borderRadius="md" bg="yellow.50">
              <Text fontSize="3xl" fontWeight="bold" color="yellow.600">
                {severityCounts.medium}
              </Text>
              <Text fontSize="sm" color="yellow.700" fontWeight="medium">
                Medium
              </Text>
            </Box>
            <Box textAlign="center" p={4} borderRadius="md" bg="green.50">
              <Text fontSize="3xl" fontWeight="bold" color="green.600">
                {severityCounts.low}
              </Text>
              <Text fontSize="sm" color="green.700" fontWeight="medium">
                Low
              </Text>
            </Box>
          </SimpleGrid>
        </VStack>
      </Box>

      {/* Status Overview */}
      <SimpleGrid columns={{ base: 2, md: 4 }} gap={4}>
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm" textAlign="center">
          <Box w={8} h={8} bg="red.500" borderRadius="md" mx="auto" mb={2} />
          <Text fontSize="2xl" fontWeight="bold" color="red.600">
            {statusCounts.open}
          </Text>
          <Text fontSize="sm" color="gray.600">Open Alerts</Text>
        </Box>

        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm" textAlign="center">
          <Box w={8} h={8} bg="orange.500" borderRadius="md" mx="auto" mb={2} />
          <Text fontSize="2xl" fontWeight="bold" color="orange.600">
            {statusCounts.investigating}
          </Text>
          <Text fontSize="sm" color="gray.600">Investigating</Text>
        </Box>

        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm" textAlign="center">
          <Box w={8} h={8} bg="green.500" borderRadius="md" mx="auto" mb={2} />
          <Text fontSize="2xl" fontWeight="bold" color="green.600">
            {statusCounts.resolved}
          </Text>
          <Text fontSize="sm" color="gray.600">Resolved</Text>
        </Box>

        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm" textAlign="center">
          <Box w={8} h={8} bg="gray.500" borderRadius="md" mx="auto" mb={2} />
          <Text fontSize="2xl" fontWeight="bold" color="gray.600">
            {alerts.length}
          </Text>
          <Text fontSize="sm" color="gray.600">Total Alerts</Text>
        </Box>
      </SimpleGrid>

      {/* Alerts Table */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
        <VStack align="stretch" gap={4}>
          <HStack justify="space-between">
            <Heading size="md">Recent Threat Alerts</Heading>
            <Text fontSize="sm" color="gray.500">
              Showing {Math.min(alerts.length, 20)} of {alerts.length} alerts
            </Text>
          </HStack>

          <Box overflowX="auto">
            {/* Custom Table Header */}
            <Box bg="gray.50" p={3} borderRadius="md" mb={2}>
              <HStack>
                <Text fontWeight="medium" fontSize="sm" flex="2">Alert</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Severity</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Status</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Agent</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Created</Text>
                <Text fontWeight="medium" fontSize="sm" flex="1">Actions</Text>
              </HStack>
            </Box>

            {/* Custom Table Body */}
            <VStack gap={0}>
              {alerts.slice(0, 20).map((alert, index) => (
                <Box key={alert.alert_id} w="100%">
                  <HStack p={3} _hover={{ bg: 'gray.50' }}>
                    <Box flex="2">
                      <VStack align="start" gap={1}>
                        <Text fontWeight="medium" fontSize="sm" overflow="hidden" textOverflow="ellipsis" whiteSpace="nowrap">
                          {alert.title}
                        </Text>
                        <Text fontSize="xs" color="gray.500" overflow="hidden" textOverflow="ellipsis" whiteSpace="nowrap">
                          {alert.description || alert.alert_type}
                        </Text>
                      </VStack>
                    </Box>
                    <Box flex="1">
                      <Badge
                        colorScheme={getSeverityColor(alert.severity)}
                        variant="solid"
                        textTransform="uppercase"
                        fontSize="xs"
                      >
                        {alert.severity}
                      </Badge>
                    </Box>
                    <Box flex="1">
                      <Badge
                        colorScheme={getStatusColor(alert.status)}
                        variant="subtle"
                        fontSize="xs"
                      >
                        {alert.status.replace('_', ' ')}
                      </Badge>
                    </Box>
                    <Box flex="1">
                      <Text fontSize="xs" fontFamily="mono" color="gray.600">
                        {alert.agent_id.slice(0, 8)}...
                      </Text>
                    </Box>
                    <Box flex="1">
                      <Text fontSize="sm" color="gray.600">
                        {formatTimeAgo(alert.created_at)}
                      </Text>
                    </Box>
                    <Box flex="1">
                      <Button size="sm" colorScheme="blue" variant="outline">
                        Investigate
                      </Button>
                    </Box>
                  </HStack>
                  {index < Math.min(alerts.length, 20) - 1 && <Box h="1px" bg="gray.100" />}
                </Box>
              ))}
            </VStack>

            {alerts.length === 0 && (
              <Box textAlign="center" py={10}>
                <Box w={12} h={12} bg="green.400" borderRadius="md" mx="auto" mb={4} />
                <Text color="gray.500" fontSize="lg">
                  No active threats detected
                </Text>
                <Text color="gray.400" fontSize="sm">
                  Your system is currently secure
                </Text>
              </Box>
            )}
          </Box>
        </VStack>
      </Box>
    </VStack>
  );
};

export default Threats;