import React from 'react';
import {
  Box,
  Heading,
  SimpleGrid,
  VStack,
  HStack,
  Text,
  Button,
} from '@chakra-ui/react';

const AdminDashboard: React.FC = () => {
  // Mock user data for demo
  const user = { username: 'admin' };

  const adminStats = {
    totalUsers: 12, // This would come from API
    activeUsers: 8,
    totalEmployees: 45,
    totalAssets: 156,
    pendingApprovals: 3,
    systemHealth: 98.5,
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Box>
          <Heading size="lg">Admin Dashboard</Heading>
          <Text color="gray.600" mt={1}>
            Welcome back, {user?.username}
          </Text>
        </Box>
        <Button colorScheme="blue">
          System Settings
        </Button>
      </HStack>

      {/* Quick Stats */}
      <SimpleGrid columns={{ base: 2, md: 3, lg: 6 }} gap={4}>
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="blue.600">
            {adminStats.totalUsers}
          </Text>
          <Text fontSize="sm" color="gray.600">Total Users</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="green.600">
            {adminStats.activeUsers}
          </Text>
          <Text fontSize="sm" color="gray.600">Active Users</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="purple.600">
            {adminStats.totalEmployees}
          </Text>
          <Text fontSize="sm" color="gray.600">Employees</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="orange.600">
            {adminStats.totalAssets}
          </Text>
          <Text fontSize="sm" color="gray.600">Assets</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="red.600">
            {adminStats.pendingApprovals}
          </Text>
          <Text fontSize="sm" color="gray.600">Pending</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="teal.600">
            {adminStats.systemHealth}%
          </Text>
          <Text fontSize="sm" color="gray.600">System Health</Text>
        </Box>
      </SimpleGrid>

      {/* Quick Actions */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <Heading size="md">Quick Actions</Heading>
          <SimpleGrid columns={{ base: 1, md: 2, lg: 4 }} gap={4}>
            <Button colorScheme="blue" variant="outline" size="lg" h="60px">
              <VStack gap={1}>
                <Text fontSize="sm" fontWeight="bold">Add User</Text>
                <Text fontSize="xs" color="gray.500">Create new user account</Text>
              </VStack>
            </Button>
            
            <Button colorScheme="green" variant="outline" size="lg" h="60px">
              <VStack gap={1}>
                <Text fontSize="sm" fontWeight="bold">Add Employee</Text>
                <Text fontSize="xs" color="gray.500">Register new employee</Text>
              </VStack>
            </Button>
            
            <Button colorScheme="purple" variant="outline" size="lg" h="60px">
              <VStack gap={1}>
                <Text fontSize="sm" fontWeight="bold">Asset Upload</Text>
                <Text fontSize="xs" color="gray.500">Upload new assets</Text>
              </VStack>
            </Button>
            
            <Button colorScheme="orange" variant="outline" size="lg" h="60px">
              <VStack gap={1}>
                <Text fontSize="sm" fontWeight="bold">Agent Update</Text>
                <Text fontSize="xs" color="gray.500">Upload agent version</Text>
              </VStack>
            </Button>
          </SimpleGrid>
        </VStack>
      </Box>

      {/* Recent Activities */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <Heading size="md">Recent Admin Activities</Heading>
          <VStack align="stretch" gap={2}>
            <Box p={3} bg="gray.50" borderRadius="md">
              <HStack justify="space-between">
                <VStack align="start" gap={0}>
                  <Text fontSize="sm" fontWeight="medium">New user registered: john.doe</Text>
                  <Text fontSize="xs" color="gray.500">Pending approval - Role: Analyst</Text>
                </VStack>
                <Text fontSize="xs" color="gray.500">2 hours ago</Text>
              </HStack>
            </Box>
            
            <Box p={3} bg="gray.50" borderRadius="md">
              <HStack justify="space-between">
                <VStack align="start" gap={0}>
                  <Text fontSize="sm" fontWeight="medium">Agent version 2.4.1 uploaded</Text>
                  <Text fontSize="xs" color="gray.500">Available for download</Text>
                </VStack>
                <Text fontSize="xs" color="gray.500">4 hours ago</Text>
              </HStack>
            </Box>
            
            <Box p={3} bg="gray.50" borderRadius="md">
              <HStack justify="space-between">
                <VStack align="start" gap={0}>
                  <Text fontSize="sm" fontWeight="medium">Employee data updated: Sarah Smith</Text>
                  <Text fontSize="xs" color="gray.500">Department changed to IT Security</Text>
                </VStack>
                <Text fontSize="xs" color="gray.500">1 day ago</Text>
              </HStack>
            </Box>
          </VStack>
        </VStack>
      </Box>
    </VStack>
  );
};

export default AdminDashboard;