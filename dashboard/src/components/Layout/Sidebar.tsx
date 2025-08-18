import React from 'react';
import {
  Box,
  VStack,
  HStack,
  Text,
  Badge,
} from '@chakra-ui/react';
import { NavLink, useLocation } from 'react-router-dom';
import {
  FiHome,
  FiShield,
  FiAlertTriangle,
  FiBarChart,
  FiSettings,
  FiServer,
} from 'react-icons/fi';
import { useSocket } from '../../contexts/SocketContext';

interface NavItem {
  label: string;
  icon: any;
  href: string;
  badge?: string | number;
}

const Sidebar: React.FC = () => {
  const location = useLocation();
  const { agents, alerts, connected } = useSocket();
  const bg = 'white';
  const borderColor = 'gray.200';

  const onlineAgents = agents.filter(agent => agent.status === 'online').length;
  const offlineAgents = agents.filter(agent => agent.status === 'offline').length;
  const criticalAlerts = alerts.filter(alert => 
    alert.severity === 'critical' && alert.status === 'open'
  ).length;

  const navItems: NavItem[] = [
    {
      label: 'Dashboard',
      icon: FiHome,
      href: '/dashboard',
    },
    {
      label: 'Agents',
      icon: FiServer,
      href: '/agents',
      badge: offlineAgents > 0 ? offlineAgents : undefined,
    },
    {
      label: 'Threats',
      icon: FiShield,
      href: '/threats',
      badge: criticalAlerts > 0 ? criticalAlerts : undefined,
    },
    {
      label: 'Alerts',
      icon: FiAlertTriangle,
      href: '/alerts',
      badge: alerts.filter(a => a.status === 'open').length || undefined,
    },
    {
      label: 'Reports',
      icon: FiBarChart,
      href: '/reports',
    },
    {
      label: 'Settings',
      icon: FiSettings,
      href: '/settings',
    },
  ];

  return (
    <Box
      w="250px"
      h="100vh"
      bg={bg}
      borderRight="1px"
      borderColor={borderColor}
      shadow="sm"
    >
      {/* Logo & Title */}
      <Box p={6} borderBottom="1px" borderColor={borderColor}>
        <HStack gap={3}>
          <Box w={8} h={8} bg="blue.500" borderRadius="md" />
          <VStack align="start" gap={0}>
            <Text fontSize="xl" fontWeight="bold" color="blue.500">
              SecureGuard
            </Text>
            <HStack gap={2}>
              <Box
                w={2}
                h={2}
                borderRadius="full"
                bg={connected ? 'green.400' : 'red.400'}
              />
              <Text fontSize="xs" color="gray.500">
                {connected ? 'Connected' : 'Disconnected'}
              </Text>
            </HStack>
          </VStack>
        </HStack>
      </Box>

      {/* Navigation */}
      <VStack gap={1} align="stretch" p={4}>
        {navItems.map((item) => {
          const isActive = location.pathname === item.href;
          
          return (
            <NavLink key={item.href} to={item.href}>
              <Box
                px={4}
                py={3}
                borderRadius="md"
                bg={isActive ? 'blue.50' : 'transparent'}
                color={isActive ? 'blue.600' : 'gray.600'}
                _hover={{
                  bg: isActive ? 'blue.50' : 'gray.50',
                  color: isActive ? 'blue.600' : 'gray.900',
                }}
                transition="all 0.2s"
                cursor="pointer"
              >
                <HStack justify="space-between">
                  <HStack gap={3}>
                    <Box w={5} h={5} bg={isActive ? 'blue.600' : 'gray.600'} borderRadius="sm" />
                    <Text fontWeight={isActive ? 'semibold' : 'medium'}>
                      {item.label}
                    </Text>
                  </HStack>
                  {item.badge && (
                    <Badge
                      colorScheme={
                        item.label === 'Threats' || item.label === 'Alerts'
                          ? 'red'
                          : item.label === 'Agents'
                          ? 'orange'
                          : 'blue'
                      }
                      borderRadius="full"
                      px={2}
                      fontSize="xs"
                    >
                      {item.badge}
                    </Badge>
                  )}
                </HStack>
              </Box>
            </NavLink>
          );
        })}
      </VStack>

      {/* Status Summary */}
      <Box p={4} mt="auto">
        <VStack align="stretch" gap={2}>
          <Text fontSize="sm" fontWeight="semibold" color="gray.700">
            System Status
          </Text>
          <HStack justify="space-between">
            <Text fontSize="xs" color="gray.500">
              Online Agents
            </Text>
            <Text fontSize="xs" fontWeight="semibold" color="green.600">
              {onlineAgents}
            </Text>
          </HStack>
          <HStack justify="space-between">
            <Text fontSize="xs" color="gray.500">
              Active Alerts
            </Text>
            <Text fontSize="xs" fontWeight="semibold" color="red.600">
              {alerts.filter(a => a.status === 'open').length}
            </Text>
          </HStack>
        </VStack>
      </Box>
    </Box>
  );
};

export default Sidebar;