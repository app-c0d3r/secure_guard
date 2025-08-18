import React from 'react';
import {
  Flex,
  HStack,
  Text,
  Badge,
  Box,
  Button,
} from '@chakra-ui/react';
import { FiBell, FiChevronDown, FiLogOut, FiSettings, FiUser } from 'react-icons/fi';
import { useAuth } from '../../contexts/AuthContext';
import { useSocket } from '../../contexts/SocketContext';

const Navbar: React.FC = () => {
  const { user, logout } = useAuth();
  const { alerts, connected } = useSocket();
  const bg = 'white';
  const borderColor = 'gray.200';

  const openAlerts = alerts.filter(alert => alert.status === 'open');
  const criticalAlerts = alerts.filter(alert => 
    alert.severity === 'critical' && alert.status === 'open'
  );

  return (
    <Flex
      as="nav"
      align="center"
      justify="space-between"
      w="full"
      px={6}
      py={4}
      bg={bg}
      borderBottom="1px"
      borderColor={borderColor}
      shadow="sm"
    >
      {/* Left side - Page title and status */}
      <HStack gap={4}>
        <Text fontSize="2xl" fontWeight="bold" color="gray.800">
          Cybersecurity Dashboard
        </Text>
        <Badge
          colorScheme={connected ? 'green' : 'red'}
          variant="subtle"
          px={2}
          py={1}
          borderRadius="md"
        >
          {connected ? 'Live Data' : 'Offline'}
        </Badge>
      </HStack>

      {/* Right side - Controls and user menu */}
      <HStack gap={4}>
        {/* Notifications */}
        <Box position="relative">
          <Box
            as="button"
            p={2}
            bg="gray.100"
            borderRadius="md"
            _hover={{ bg: 'gray.200' }}
            position="relative"
          >
            <Box w={5} h={5} bg="gray.600" borderRadius="sm" />
            {openAlerts.length > 0 && (
              <Badge
                position="absolute"
                top="-1"
                right="-1"
                colorScheme={criticalAlerts.length > 0 ? 'red' : 'orange'}
                borderRadius="full"
                boxSize={4}
                fontSize="10px"
              >
                {openAlerts.length}
              </Badge>
            )}
          </Box>
        </Box>

        {/* User menu */}
        <Button variant="ghost" onClick={logout}>
          <HStack gap={3}>
            <Box w={8} h={8} bg="blue.500" borderRadius="full" />
            <Box textAlign="left">
              <Text fontSize="sm" fontWeight="medium">
                {user?.username}
              </Text>
              <Text fontSize="xs" color="gray.500">
                Administrator
              </Text>
            </Box>
          </HStack>
        </Button>
      </HStack>
    </Flex>
  );
};

export default Navbar;