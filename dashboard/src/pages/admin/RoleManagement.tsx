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

interface Permission {
  id: string;
  name: string;
  description: string;
  category: 'Dashboard' | 'Users' | 'Assets' | 'Reports' | 'Settings' | 'Admin';
}

interface Role {
  id: string;
  name: string;
  description: string;
  level: 'Low' | 'Medium' | 'High' | 'Critical';
  userCount: number;
  permissions: string[];
  isSystemRole: boolean;
  createdAt: string;
  lastModified: string;
}

const RoleManagement: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [showCreateRole, setShowCreateRole] = useState(false);
  const [newRole, setNewRole] = useState({
    name: '',
    description: '',
    permissions: [] as string[],
  });

  // Mock permissions data
  const [permissions] = useState<Permission[]>([
    { id: 'dashboard_view', name: 'View Dashboard', description: 'Access main dashboard', category: 'Dashboard' },
    { id: 'dashboard_manage', name: 'Manage Dashboard', description: 'Configure dashboard settings', category: 'Dashboard' },
    { id: 'users_view', name: 'View Users', description: 'View user accounts', category: 'Users' },
    { id: 'users_create', name: 'Create Users', description: 'Create new user accounts', category: 'Users' },
    { id: 'users_edit', name: 'Edit Users', description: 'Modify existing user accounts', category: 'Users' },
    { id: 'users_delete', name: 'Delete Users', description: 'Remove user accounts', category: 'Users' },
    { id: 'assets_view', name: 'View Assets', description: 'View asset inventory', category: 'Assets' },
    { id: 'assets_manage', name: 'Manage Assets', description: 'Add, edit, or remove assets', category: 'Assets' },
    { id: 'reports_view', name: 'View Reports', description: 'Access security reports', category: 'Reports' },
    { id: 'reports_export', name: 'Export Reports', description: 'Export reports to external formats', category: 'Reports' },
    { id: 'settings_view', name: 'View Settings', description: 'Access system settings', category: 'Settings' },
    { id: 'settings_modify', name: 'Modify Settings', description: 'Change system configuration', category: 'Settings' },
    { id: 'admin_full', name: 'Full Admin Access', description: 'Complete administrative control', category: 'Admin' },
  ]);

  // Mock roles data
  const [roles] = useState<Role[]>([
    {
      id: '1',
      name: 'Super Admin',
      description: 'Full system access with all permissions',
      level: 'Critical',
      userCount: 2,
      permissions: ['admin_full'],
      isSystemRole: true,
      createdAt: '2024-01-01',
      lastModified: '2024-01-01',
    },
    {
      id: '2',
      name: 'Admin',
      description: 'Administrative access with most permissions',
      level: 'High',
      userCount: 5,
      permissions: ['dashboard_view', 'dashboard_manage', 'users_view', 'users_create', 'users_edit', 'assets_view', 'assets_manage', 'reports_view', 'reports_export', 'settings_view'],
      isSystemRole: true,
      createdAt: '2024-01-01',
      lastModified: '2024-01-15',
    },
    {
      id: '3',
      name: 'Manager',
      description: 'Management level access with user and asset oversight',
      level: 'Medium',
      userCount: 8,
      permissions: ['dashboard_view', 'users_view', 'assets_view', 'assets_manage', 'reports_view', 'reports_export'],
      isSystemRole: true,
      createdAt: '2024-01-01',
      lastModified: '2024-01-10',
    },
    {
      id: '4',
      name: 'Supervisor',
      description: 'Supervisory access with limited management capabilities',
      level: 'Medium',
      userCount: 12,
      permissions: ['dashboard_view', 'users_view', 'assets_view', 'reports_view'],
      isSystemRole: true,
      createdAt: '2024-01-01',
      lastModified: '2024-01-05',
    },
    {
      id: '5',
      name: 'Analyst',
      description: 'Read-only access for security analysis',
      level: 'Low',
      userCount: 25,
      permissions: ['dashboard_view', 'reports_view'],
      isSystemRole: true,
      createdAt: '2024-01-01',
      lastModified: '2024-01-01',
    },
  ]);

  const categories = ['Dashboard', 'Users', 'Assets', 'Reports', 'Settings', 'Admin'];

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'Critical': return 'red';
      case 'High': return 'orange';
      case 'Medium': return 'yellow';
      case 'Low': return 'green';
      default: return 'gray';
    }
  };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'Dashboard': return 'blue';
      case 'Users': return 'green';
      case 'Assets': return 'purple';
      case 'Reports': return 'orange';
      case 'Settings': return 'teal';
      case 'Admin': return 'red';
      default: return 'gray';
    }
  };

  const filteredRoles = roles.filter(role => {
    const matchesSearch = role.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         role.description.toLowerCase().includes(searchTerm.toLowerCase());
    return matchesSearch;
  });

  const filteredPermissions = permissions.filter(permission => {
    const matchesCategory = selectedCategory === 'all' || permission.category === selectedCategory;
    return matchesCategory;
  });

  const handleCreateRole = () => {
    console.log('Creating role:', newRole);
    setShowCreateRole(false);
    setNewRole({
      name: '',
      description: '',
      permissions: [],
    });
  };

  const togglePermission = (permissionId: string) => {
    setNewRole(prev => ({
      ...prev,
      permissions: prev.permissions.includes(permissionId)
        ? prev.permissions.filter(id => id !== permissionId)
        : [...prev.permissions, permissionId]
    }));
  };

  const getPermissionName = (permissionId: string) => {
    const permission = permissions.find(p => p.id === permissionId);
    return permission ? permission.name : permissionId;
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Box>
          <Heading size="lg">Role & Permissions Management</Heading>
          <Text color="gray.600" mt={1}>
            Manage user roles and their access permissions across the system
          </Text>
        </Box>
        <Button colorScheme="blue" onClick={() => setShowCreateRole(true)}>
          Create New Role
        </Button>
      </HStack>

      {/* Statistics */}
      <SimpleGrid columns={{ base: 2, md: 4 }} gap={4}>
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="blue.600">
            {roles.length}
          </Text>
          <Text fontSize="sm" color="gray.600">Total Roles</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="green.600">
            {permissions.length}
          </Text>
          <Text fontSize="sm" color="gray.600">Permissions</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="orange.600">
            {roles.reduce((sum, role) => sum + role.userCount, 0)}
          </Text>
          <Text fontSize="sm" color="gray.600">Users Assigned</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="purple.600">
            {roles.filter(r => !r.isSystemRole).length}
          </Text>
          <Text fontSize="sm" color="gray.600">Custom Roles</Text>
        </Box>
      </SimpleGrid>

      {/* Create Role Form */}
      {showCreateRole && (
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          <VStack align="stretch" gap={4}>
            <HStack justify="space-between">
              <Heading size="md">Create New Role</Heading>
              <Button variant="outline" onClick={() => setShowCreateRole(false)}>
                Cancel
              </Button>
            </HStack>
            
            <SimpleGrid columns={{ base: 1, md: 2 }} gap={4}>
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Role Name
                </Text>
                <Input
                  placeholder="e.g., Security Specialist"
                  value={newRole.name}
                  onChange={(e) => setNewRole({...newRole, name: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Description
                </Text>
                <Input
                  placeholder="Brief description of the role..."
                  value={newRole.description}
                  onChange={(e) => setNewRole({...newRole, description: e.target.value})}
                  size="sm"
                />
              </Box>
            </SimpleGrid>

            {/* Permission Categories */}
            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Permissions by Category
              </Text>
              <HStack wrap="wrap" gap={2} mb={4}>
                <Button
                  size="sm"
                  variant={selectedCategory === 'all' ? 'solid' : 'outline'}
                  colorScheme={selectedCategory === 'all' ? 'blue' : 'gray'}
                  onClick={() => setSelectedCategory('all')}
                >
                  All Categories
                </Button>
                {categories.map(category => (
                  <Button
                    key={category}
                    size="sm"
                    variant={selectedCategory === category ? 'solid' : 'outline'}
                    colorScheme={selectedCategory === category ? 'blue' : 'gray'}
                    onClick={() => setSelectedCategory(category)}
                  >
                    {category}
                  </Button>
                ))}
              </HStack>

              {/* Permissions Grid */}
              <SimpleGrid columns={{ base: 1, md: 2, lg: 3 }} gap={3}>
                {filteredPermissions.map(permission => (
                  <Box
                    key={permission.id}
                    p={3}
                    border="1px"
                    borderColor={newRole.permissions.includes(permission.id) ? 'blue.300' : 'gray.200'}
                    borderRadius="md"
                    bg={newRole.permissions.includes(permission.id) ? 'blue.50' : 'white'}
                    cursor="pointer"
                    onClick={() => togglePermission(permission.id)}
                    _hover={{ borderColor: 'blue.300', bg: 'blue.50' }}
                  >
                    <VStack align="start" gap={1}>
                      <HStack justify="space-between" w="100%">
                        <Text fontSize="sm" fontWeight="medium">
                          {permission.name}
                        </Text>
                        <Badge
                          colorScheme={getCategoryColor(permission.category)}
                          variant="subtle"
                          fontSize="xs"
                        >
                          {permission.category}
                        </Badge>
                      </HStack>
                      <Text fontSize="xs" color="gray.500">
                        {permission.description}
                      </Text>
                    </VStack>
                  </Box>
                ))}
              </SimpleGrid>
            </Box>
            
            <HStack justify="end">
              <Text fontSize="sm" color="gray.500" mr={4}>
                {newRole.permissions.length} permissions selected
              </Text>
              <Button colorScheme="blue" onClick={handleCreateRole}>
                Create Role
              </Button>
            </HStack>
          </VStack>
        </Box>
      )}

      {/* Roles List */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <HStack justify="space-between">
            <Heading size="md">System Roles ({filteredRoles.length})</Heading>
            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Search Roles
              </Text>
              <Input
                placeholder="Search by name or description..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                size="sm"
                w="300px"
              />
            </Box>
          </HStack>
          
          {/* Table Header */}
          <Box bg="gray.50" p={3} borderRadius="md">
            <HStack>
              <Text fontWeight="medium" fontSize="sm" flex="2">Role</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Access Level</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Users</Text>
              <Text fontWeight="medium" fontSize="sm" flex="2">Permissions</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Actions</Text>
            </HStack>
          </Box>
          
          {/* Table Body */}
          <VStack gap={0}>
            {filteredRoles.map((role, index) => (
              <Box key={role.id} w="100%">
                <HStack p={3} _hover={{ bg: 'gray.50' }}>
                  <Box flex="2">
                    <VStack align="start" gap={1}>
                      <HStack>
                        <Text fontWeight="medium" fontSize="sm">
                          {role.name}
                        </Text>
                        {role.isSystemRole && (
                          <Badge
                            colorScheme="gray"
                            variant="subtle"
                            fontSize="xs"
                          >
                            System
                          </Badge>
                        )}
                      </HStack>
                      <Text fontSize="xs" color="gray.500">
                        {role.description}
                      </Text>
                      <Text fontSize="xs" color="gray.400">
                        Created: {role.createdAt} • Modified: {role.lastModified}
                      </Text>
                    </VStack>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getLevelColor(role.level)}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {role.level}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Text fontSize="sm" color="gray.600">
                      {role.userCount} users
                    </Text>
                  </Box>
                  
                  <Box flex="2">
                    <HStack wrap="wrap" gap={1}>
                      {role.permissions.slice(0, 3).map(permissionId => (
                        <Badge
                          key={permissionId}
                          colorScheme="blue"
                          variant="subtle"
                          fontSize="xs"
                        >
                          {getPermissionName(permissionId)}
                        </Badge>
                      ))}
                      {role.permissions.length > 3 && (
                        <Badge
                          colorScheme="gray"
                          variant="subtle"
                          fontSize="xs"
                        >
                          +{role.permissions.length - 3} more
                        </Badge>
                      )}
                    </HStack>
                  </Box>
                  
                  <Box flex="1">
                    <HStack gap={1}>
                      <Button size="xs" variant="outline" colorScheme="blue">
                        View
                      </Button>
                      {!role.isSystemRole && (
                        <>
                          <Button size="xs" variant="outline" colorScheme="gray">
                            Edit
                          </Button>
                          <Button size="xs" variant="outline" colorScheme="red">
                            Delete
                          </Button>
                        </>
                      )}
                    </HStack>
                  </Box>
                </HStack>
                {index < filteredRoles.length - 1 && <Box h="1px" bg="gray.100" />}
              </Box>
            ))}
          </VStack>
          
          {filteredRoles.length === 0 && (
            <Box textAlign="center" py={8}>
              <Text color="gray.500" fontSize="lg">
                No roles found
              </Text>
              <Text color="gray.400" fontSize="sm">
                Try adjusting your search criteria
              </Text>
            </Box>
          )}
        </VStack>
      </Box>

      {/* Access Control Matrix */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <Heading size="md">Role Access Matrix</Heading>
          <Text fontSize="sm" color="gray.600">
            Quick overview of role permissions across different system areas
          </Text>
          
          <Box overflowX="auto">
            <Box minW="800px">
              {/* Matrix Header */}
              <HStack bg="gray.50" p={2} borderRadius="md" mb={2}>
                <Text fontWeight="medium" fontSize="sm" w="150px">Role</Text>
                {categories.map(category => (
                  <Text key={category} fontWeight="medium" fontSize="sm" flex="1" textAlign="center">
                    {category}
                  </Text>
                ))}
              </HStack>
              
              {/* Matrix Body */}
              <VStack gap={1}>
                {roles.map(role => (
                  <HStack key={role.id} p={2} _hover={{ bg: 'gray.50' }} borderRadius="md">
                    <Box w="150px">
                      <Text fontSize="sm" fontWeight="medium">{role.name}</Text>
                    </Box>
                    {categories.map(category => {
                      const hasPermission = role.permissions.some(permId => {
                        const perm = permissions.find(p => p.id === permId);
                        return perm?.category === category || permId === 'admin_full';
                      });
                      return (
                        <Box key={category} flex="1" textAlign="center">
                          <Box
                            w="20px"
                            h="20px"
                            borderRadius="full"
                            bg={hasPermission ? 'green.500' : 'gray.200'}
                            mx="auto"
                          />
                        </Box>
                      );
                    })}
                  </HStack>
                ))}
              </VStack>
            </Box>
          </Box>
        </VStack>
      </Box>
    </VStack>
  );
};

export default RoleManagement;