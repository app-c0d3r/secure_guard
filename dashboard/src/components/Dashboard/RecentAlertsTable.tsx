import React from 'react';
import {
  Badge,
  Text,
  VStack,
  HStack,
  Box,
} from '@chakra-ui/react';
import { FiClock, FiAlertTriangle } from 'react-icons/fi';

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

interface RecentAlertsTableProps {
  alerts: ThreatAlert[];
}

const RecentAlertsTable: React.FC<RecentAlertsTableProps> = ({ alerts }) => {
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

  if (alerts.length === 0) {
    return (
      <VStack gap={4} py={8}>
        <Box w={8} h={8} bg="gray.400" borderRadius="md" />
        <Text color="gray.500" textAlign="center">
          No recent alerts found
        </Text>
      </VStack>
    );
  }

  return (
    <Box overflowX="auto">
      <Box bg="white" border="1px" borderColor="gray.200" borderRadius="md">
        {/* Header */}
        <Box bg="gray.50" p={3} borderTopRadius="md">
          <HStack>
            <Text fontWeight="medium" fontSize="sm" flex="2">Alert</Text>
            <Text fontWeight="medium" fontSize="sm" flex="1">Severity</Text>
            <Text fontWeight="medium" fontSize="sm" flex="1">Status</Text>
            <Text fontWeight="medium" fontSize="sm" flex="1">Agent</Text>
            <Text fontWeight="medium" fontSize="sm" flex="1">Time</Text>
          </HStack>
        </Box>
        
        {/* Body */}
        <VStack gap={0}>
          {alerts.map((alert, index) => (
            <Box key={alert.alert_id} w="100%">
              <HStack p={3} w="100%" _hover={{ bg: 'gray.50' }}>
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
                    variant="subtle"
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
                  <Text fontSize="xs" fontFamily="mono">
                    {alert.agent_id.slice(0, 8)}...
                  </Text>
                </Box>
                <Box flex="1">
                  <HStack gap={1}>
                    <Box w={3} h={3} bg="gray.400" borderRadius="sm" />
                    <Text fontSize="xs" color="gray.500">
                      {formatTimeAgo(alert.created_at)}
                    </Text>
                  </HStack>
                </Box>
              </HStack>
              {index < alerts.length - 1 && <Box h="1px" bg="gray.100" />}
            </Box>
          ))}
        </VStack>
      </Box>
    </Box>
  );
};

export default RecentAlertsTable;