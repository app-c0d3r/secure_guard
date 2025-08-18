import React, { useState } from 'react';
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Button,
  Input,
  Badge,
  SimpleGrid,
} from '@chakra-ui/react';

interface User {
  id: string;
  username: string;
  email: string;
  firstName: string;
  lastName: string;
  role: string;
  status: 'active' | 'inactive' | 'pending';
  lastLogin: string;
  createdAt: string;
  department: string;
}

const UserManagement: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedRole, setSelectedRole] = useState('all');
  const [showAddUser, setShowAddUser] = useState(false);
  const [newUser, setNewUser] = useState({
    username: '',
    email: '',
    firstName: '',
    lastName: '',
    role: 'analyst',
    department: '',
  });

  // Mock data - would come from API
  const [users] = useState<User[]>([
    {
      id: '1',
      username: 'john.doe',
      email: 'john.doe@company.com',
      firstName: 'John',
      lastName: 'Doe',
      role: 'Admin',
      status: 'active',
      lastLogin: '2024-01-20 14:30',
      createdAt: '2024-01-15',
      department: 'IT Security',
    },
    {
      id: '2',
      username: 'jane.smith',
      email: 'jane.smith@company.com',
      firstName: 'Jane',
      lastName: 'Smith',
      role: 'Analyst',
      status: 'active',
      lastLogin: '2024-01-20 09:15',
      createdAt: '2024-01-10',
      department: 'Cybersecurity',
    },
    {
      id: '3',
      username: 'mike.wilson',
      email: 'mike.wilson@company.com',
      firstName: 'Mike',
      lastName: 'Wilson',
      role: 'Supervisor',
      status: 'pending',
      lastLogin: 'Never',
      createdAt: '2024-01-19',
      department: 'IT Operations',
    },
  ]);

  const roles = ['Super Admin', 'Admin', 'Manager', 'Supervisor', 'Analyst'];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'green';
      case 'inactive': return 'red';
      case 'pending': return 'orange';
      default: return 'gray';
    }
  };

  const getRoleColor = (role: string) => {
    switch (role) {
      case 'Super Admin': return 'purple';
      case 'Admin': return 'blue';
      case 'Manager': return 'teal';
      case 'Supervisor': return 'orange';
      case 'Analyst': return 'green';
      default: return 'gray';
    }
  };

  const filteredUsers = users.filter(user => {
    const matchesSearch = user.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         user.firstName.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         user.lastName.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesRole = selectedRole === 'all' || user.role === selectedRole;
    return matchesSearch && matchesRole;
  });

  const handleAddUser = () => {
    // Here you would typically call an API to create the user
    console.log('Adding user:', newUser);
    setShowAddUser(false);
    setNewUser({
      username: '',
      email: '',
      firstName: '',
      lastName: '',
      role: 'analyst',
      department: '',
    });
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Box>
          <Heading size="lg">User Management</Heading>
          <Text color="gray.600" mt={1}>
            Manage user accounts, roles, and permissions
          </Text>
        </Box>
        <Button colorScheme="blue" onClick={() => setShowAddUser(true)}>
          Add New User
        </Button>
      </HStack>

      {/* Filters */}
      <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
        <HStack gap={4}>
          <Box flex="1">
            <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
              Search Users
            </Text>
            <Input
              placeholder="Search by name, username, or email..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              size="sm"
            />
          </Box>
          
          <Box>
            <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
              Filter by Role
            </Text>
            <Button
              size="sm"
              variant={selectedRole === 'all' ? 'solid' : 'outline'}
              colorScheme={selectedRole === 'all' ? 'blue' : 'gray'}
              onClick={() => setSelectedRole('all')}
              mr={2}
            >
              All Roles
            </Button>
            {roles.map(role => (
              <Button
                key={role}
                size="sm"
                variant={selectedRole === role ? 'solid' : 'outline'}
                colorScheme={selectedRole === role ? 'blue' : 'gray'}
                onClick={() => setSelectedRole(role)}
                mr={2}
              >
                {role}
              </Button>
            ))}
          </Box>
        </HStack>
      </Box>

      {/* Add User Form */}
      {showAddUser && (
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          <VStack align="stretch" gap={4}>
            <HStack justify="space-between">
              <Heading size="md">Add New User</Heading>
              <Button variant="outline" onClick={() => setShowAddUser(false)}>
                Cancel
              </Button>
            </HStack>
            
            <SimpleGrid columns={{ base: 1, md: 2 }} gap={4}>
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  First Name
                </Text>
                <Input
                  value={newUser.firstName}
                  onChange={(e) => setNewUser({...newUser, firstName: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Last Name
                </Text>
                <Input
                  value={newUser.lastName}
                  onChange={(e) => setNewUser({...newUser, lastName: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Username
                </Text>
                <Input
                  value={newUser.username}
                  onChange={(e) => setNewUser({...newUser, username: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Email
                </Text>
                <Input
                  type="email"
                  value={newUser.email}
                  onChange={(e) => setNewUser({...newUser, email: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Department
                </Text>
                <Input
                  value={newUser.department}
                  onChange={(e) => setNewUser({...newUser, department: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Role
                </Text>
                <HStack wrap="wrap" gap={2}>
                  {roles.map(role => (
                    <Button
                      key={role}
                      size="sm"
                      variant={newUser.role === role.toLowerCase().replace(' ', '_') ? 'solid' : 'outline'}
                      colorScheme={newUser.role === role.toLowerCase().replace(' ', '_') ? 'blue' : 'gray'}
                      onClick={() => setNewUser({...newUser, role: role.toLowerCase().replace(' ', '_')})}
                    >
                      {role}
                    </Button>
                  ))}
                </HStack>
              </Box>
            </SimpleGrid>
            
            <HStack justify="end">
              <Button colorScheme="blue" onClick={handleAddUser}>
                Create User
              </Button>
            </HStack>
          </VStack>
        </Box>
      )}

      {/* Users Table */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <HStack justify="space-between">
            <Heading size="md">Users ({filteredUsers.length})</Heading>
            <Text fontSize="sm" color="gray.500">
              Total: {users.length} users
            </Text>
          </HStack>
          
          {/* Table Header */}
          <Box bg="gray.50" p={3} borderRadius="md">
            <HStack>
              <Text fontWeight="medium" fontSize="sm" flex="2">User</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Role</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Department</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Status</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Last Login</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Actions</Text>
            </HStack>
          </Box>
          
          {/* Table Body */}
          <VStack gap={0}>
            {filteredUsers.map((user, index) => (
              <Box key={user.id} w="100%">
                <HStack p={3} _hover={{ bg: 'gray.50' }}>
                  <Box flex="2">
                    <VStack align="start" gap={1}>
                      <Text fontWeight="medium" fontSize="sm">
                        {user.firstName} {user.lastName}
                      </Text>
                      <Text fontSize="xs" color="gray.500">
                        @{user.username} • {user.email}
                      </Text>
                    </VStack>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getRoleColor(user.role)}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {user.role}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Text fontSize="sm">{user.department}</Text>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getStatusColor(user.status)}
                      variant="subtle"
                      fontSize="xs"
                      textTransform="capitalize"
                    >
                      {user.status}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Text fontSize="sm" color="gray.600">
                      {user.lastLogin}
                    </Text>
                  </Box>
                  
                  <Box flex="1">
                    <HStack gap={2}>
                      <Button size="xs" variant="outline" colorScheme="blue">
                        Edit
                      </Button>
                      <Button size="xs" variant="outline" colorScheme="red">
                        Delete
                      </Button>
                    </HStack>
                  </Box>
                </HStack>
                {index < filteredUsers.length - 1 && <Box h="1px" bg="gray.100" />}
              </Box>
            ))}
          </VStack>
          
          {filteredUsers.length === 0 && (
            <Box textAlign="center" py={8}>
              <Text color="gray.500" fontSize="lg">
                No users found
              </Text>
              <Text color="gray.400" fontSize="sm">
                Try adjusting your search criteria
              </Text>
            </Box>
          )}
        </VStack>
      </Box>
    </VStack>
  );
};

export default UserManagement;