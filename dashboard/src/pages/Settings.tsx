import React from 'react';
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Button,
  SimpleGrid,
} from '@chakra-ui/react';

// Custom Switch component
const CustomSwitch = ({ id, defaultChecked = false }: { id: string; defaultChecked?: boolean }) => {
  const [isChecked, setIsChecked] = React.useState(defaultChecked);
  
  return (
    <button
      type="button"
      id={id}
      onClick={() => setIsChecked(!isChecked)}
      style={{
        width: '44px',
        height: '24px',
        backgroundColor: isChecked ? '#3182CE' : '#CBD5E0',
        borderRadius: '12px',
        position: 'relative',
        transition: 'background-color 0.2s',
        border: 'none',
        cursor: 'pointer',
        outline: 'none',
      }}
      onFocus={(e) => (e.currentTarget.style.boxShadow = '0 0 0 3px rgba(49, 130, 206, 0.6)')}
      onBlur={(e) => (e.currentTarget.style.boxShadow = 'none')}
    >
      <Box
        w="20px"
        h="20px"
        bg="white"
        borderRadius="full"
        position="absolute"
        top="2px"
        left={isChecked ? "22px" : "2px"}
        transition="left 0.2s"
        boxShadow="sm"
      />
    </button>
  );
};
import { FiSettings, FiShield, FiBell, FiUsers, FiDatabase } from 'react-icons/fi';

const Settings: React.FC = () => {
  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Heading size="lg">System Settings</Heading>
        <Button colorScheme="blue">
          Save Changes
        </Button>
      </HStack>

      {/* Settings Categories */}
      <SimpleGrid columns={{ base: 1, lg: 2 }} gap={6}>
        {/* Security Settings */}
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
          <VStack align="stretch" gap={4}>
            <HStack>
              <Box w={6} h={6} bg="blue.500" borderRadius="md" />
              <Heading size="md">Security Configuration</Heading>
            </HStack>
            
            <VStack align="stretch" gap={3}>
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Automatic Threat Isolation
                </Text>
                <CustomSwitch id="auto-isolation" defaultChecked />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Real-time Monitoring
                </Text>
                <CustomSwitch id="real-time" defaultChecked />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Advanced Threat Detection
                </Text>
                <CustomSwitch id="advanced-detection" defaultChecked />
              </HStack>
            </VStack>
          </VStack>
        </Box>

        {/* Notification Settings */}
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
          <VStack align="stretch" gap={4}>
            <HStack>
              <Box w={6} h={6} bg="orange.500" borderRadius="md" />
              <Heading size="md">Notification Preferences</Heading>
            </HStack>
            
            <VStack align="stretch" gap={3}>
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Email Alerts
                </Text>
                <CustomSwitch id="email-alerts" defaultChecked />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  SMS for Critical Threats
                </Text>
                <CustomSwitch id="sms-critical" />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Slack Integration
                </Text>
                <CustomSwitch id="slack-integration" />
              </HStack>
            </VStack>
          </VStack>
        </Box>

        {/* User Management */}
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
          <VStack align="stretch" gap={4}>
            <HStack>
              <Box w={6} h={6} bg="green.500" borderRadius="md" />
              <Heading size="md">User Management</Heading>
            </HStack>
            
            <VStack align="stretch" gap={3}>
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Require Multi-Factor Authentication
                </Text>
                <CustomSwitch id="mfa-required" defaultChecked />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Automatic Session Timeout
                </Text>
                <CustomSwitch id="session-timeout" defaultChecked />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Enhanced Audit Logging
                </Text>
                <CustomSwitch id="audit-logging" defaultChecked />
              </HStack>
            </VStack>
          </VStack>
        </Box>

        {/* Data Management */}
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
          <VStack align="stretch" gap={4}>
            <HStack>
              <Box w={6} h={6} bg="purple.500" borderRadius="md" />
              <Heading size="md">Data Management</Heading>
            </HStack>
            
            <VStack align="stretch" gap={3}>
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Extended Data Retention (1 Year)
                </Text>
                <CustomSwitch id="data-retention" defaultChecked />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Automatic Backups
                </Text>
                <CustomSwitch id="auto-backup" defaultChecked />
              </HStack>
              
              <HStack justify="space-between" align="center">
                <Text fontSize="sm">
                  Database Encryption at Rest
                </Text>
                <CustomSwitch id="encryption" defaultChecked />
              </HStack>
            </VStack>
          </VStack>
        </Box>
      </SimpleGrid>

      {/* System Information */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
        <VStack align="stretch" gap={4}>
          <HStack>
            <Box w={6} h={6} bg="gray.500" borderRadius="md" />
            <Heading size="md">System Information</Heading>
          </HStack>
          
          <SimpleGrid columns={{ base: 1, md: 3 }} gap={6}>
            <VStack align="start">
              <Text fontSize="sm" fontWeight="bold" color="gray.700">
                Platform Version
              </Text>
              <Text fontSize="sm" color="gray.600">
                SecureGuard v2.4.0
              </Text>
            </VStack>
            
            <VStack align="start">
              <Text fontSize="sm" fontWeight="bold" color="gray.700">
                Database Status
              </Text>
              <Text fontSize="sm" color="green.600">
                Connected - PostgreSQL
              </Text>
            </VStack>
            
            <VStack align="start">
              <Text fontSize="sm" fontWeight="bold" color="gray.700">
                Last Updated
              </Text>
              <Text fontSize="sm" color="gray.600">
                2024-01-15 14:30:22 UTC
              </Text>
            </VStack>
          </SimpleGrid>
        </VStack>
      </Box>
    </VStack>
  );
};

export default Settings;