import React, { useState } from 'react';
import {
  Box,
  HStack,
  VStack,
  Text,
  Button,
  Heading,
} from '@chakra-ui/react';
import AdminDashboard from './admin/AdminDashboard';
import UserProfile from './admin/UserProfile';
import UserManagement from './admin/UserManagement';
import EmployeeManagement from './admin/EmployeeManagement';
import AssetManagement from './admin/AssetManagement';
import AgentManagement from './admin/AgentManagement';
import RoleManagement from './admin/RoleManagement';
import UserSettings from './admin/UserSettings';

interface AdminProps {
  onBackToDashboard?: () => void;
}

const Admin: React.FC<AdminProps> = ({ onBackToDashboard }) => {
  const [activeTab, setActiveTab] = useState('dashboard');

  const adminTabs = [
    { id: 'dashboard', label: 'Dashboard', icon: '📊' },
    { id: 'profile', label: 'My Profile', icon: '👤' },
    { id: 'users', label: 'User Management', icon: '👥' },
    { id: 'employees', label: 'Employees', icon: '🏢' },
    { id: 'assets', label: 'Assets', icon: '💻' },
    { id: 'agents', label: 'Agents', icon: '🔧' },
    { id: 'roles', label: 'Roles & Permissions', icon: '🔐' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
  ];

  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return <AdminDashboard />;
      case 'profile':
        return <UserProfile />;
      case 'users':
        return <UserManagement />;
      case 'employees':
        return <EmployeeManagement />;
      case 'assets':
        return <AssetManagement />;
      case 'agents':
        return <AgentManagement />;
      case 'roles':
        return <RoleManagement />;
      case 'settings':
        return <UserSettings />;
      default:
        return <AdminDashboard />;
    }
  };

  return (
    <Box h="100vh" bg="gray.50">
      <VStack gap={0} h="100%">
        {/* Admin Header */}
        <Box bg="white" w="100%" p={4} borderBottom="1px" borderColor="gray.200">
          <HStack justify="space-between">
            <Box>
              <Heading size="lg" color="blue.600">SecureGuard Admin</Heading>
              <Text fontSize="sm" color="gray.600">
                Administrative Control Panel
              </Text>
            </Box>
            <HStack gap={2}>
              {onBackToDashboard && (
                <Button size="sm" variant="outline" onClick={onBackToDashboard} mr={2}>
                  ← Back to Dashboard
                </Button>
              )}
              <Text fontSize="sm" color="gray.500">
                Admin Access Level
              </Text>
              <Box
                px={3}
                py={1}
                bg="red.100"
                color="red.700"
                borderRadius="full"
                fontSize="xs"
                fontWeight="bold"
              >
                SUPER ADMIN
              </Box>
            </HStack>
          </HStack>
        </Box>

        <HStack gap={0} flex="1" w="100%" align="stretch">
          {/* Sidebar Navigation */}
          <Box bg="white" w="280px" borderRight="1px" borderColor="gray.200" p={4}>
            <VStack gap={2} align="stretch">
              <Text fontSize="sm" fontWeight="bold" color="gray.700" mb={2} textTransform="uppercase">
                Admin Menu
              </Text>
              {adminTabs.map(tab => (
                <Button
                  key={tab.id}
                  variant={activeTab === tab.id ? 'solid' : 'ghost'}
                  colorScheme={activeTab === tab.id ? 'blue' : 'gray'}
                  justifyContent="flex-start"
                  size="sm"
                  onClick={() => setActiveTab(tab.id)}
                  _hover={{
                    bg: activeTab === tab.id ? 'blue.600' : 'gray.100',
                  }}
                >
                  <HStack w="100%" justify="flex-start">
                    <Text>{tab.icon}</Text>
                    <Text>{tab.label}</Text>
                  </HStack>
                </Button>
              ))}
              
              {/* Admin Info Section */}
              <Box mt={6} p={3} bg="blue.50" borderRadius="md" border="1px" borderColor="blue.200">
                <VStack gap={2} align="stretch">
                  <Text fontSize="xs" fontWeight="bold" color="blue.700" textTransform="uppercase">
                    Admin Information
                  </Text>
                  <VStack gap={1} align="start">
                    <HStack justify="space-between" w="100%">
                      <Text fontSize="xs" color="blue.600">Active Users:</Text>
                      <Text fontSize="xs" fontWeight="bold" color="blue.700">12</Text>
                    </HStack>
                    <HStack justify="space-between" w="100%">
                      <Text fontSize="xs" color="blue.600">Total Assets:</Text>
                      <Text fontSize="xs" fontWeight="bold" color="blue.700">156</Text>
                    </HStack>
                    <HStack justify="space-between" w="100%">
                      <Text fontSize="xs" color="blue.600">System Health:</Text>
                      <Text fontSize="xs" fontWeight="bold" color="green.600">98.5%</Text>
                    </HStack>
                  </VStack>
                </VStack>
              </Box>

              {/* Quick Actions */}
              <Box mt={4}>
                <Text fontSize="xs" fontWeight="bold" color="gray.700" mb={2} textTransform="uppercase">
                  Quick Actions
                </Text>
                <VStack gap={1} align="stretch">
                  <Button size="xs" colorScheme="green" variant="outline">
                    Export Data
                  </Button>
                  <Button size="xs" colorScheme="orange" variant="outline">
                    System Backup
                  </Button>
                  <Button size="xs" colorScheme="red" variant="outline">
                    Security Audit
                  </Button>
                </VStack>
              </Box>
            </VStack>
          </Box>

          {/* Main Content Area */}
          <Box flex="1" p={6} overflowY="auto">
            {renderContent()}
          </Box>
        </HStack>
      </VStack>
    </Box>
  );
};

export default Admin;