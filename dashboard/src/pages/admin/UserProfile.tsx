import React, { useState } from 'react';
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Button,
  Input,
  SimpleGrid,
  Badge,
} from '@chakra-ui/react';

const UserProfile: React.FC = () => {
  // Mock user data for demo
  const user = { username: 'admin' };
  const [isEditing, setIsEditing] = useState(false);
  const [formData, setFormData] = useState({
    username: user?.username || '',
    email: 'admin@secureguard.com',
    firstName: 'Admin',
    lastName: 'User',
    department: 'IT Security',
    role: 'Super Admin',
    phone: '+1 (555) 123-4567',
    location: 'San Francisco, CA',
  });

  const handleInputChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  const handleSave = () => {
    // Here you would typically call an API to update the user profile
    console.log('Saving profile:', formData);
    setIsEditing(false);
  };

  const handleCancel = () => {
    // Reset form data to original values
    setFormData({
      username: user?.username || '',
      email: 'admin@secureguard.com',
      firstName: 'Admin',
      lastName: 'User',
      department: 'IT Security',
      role: 'Super Admin',
      phone: '+1 (555) 123-4567',
      location: 'San Francisco, CA',
    });
    setIsEditing(false);
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Box>
          <Heading size="lg">User Profile</Heading>
          <Text color="gray.600" mt={1}>
            Manage your personal information and preferences
          </Text>
        </Box>
        <HStack gap={2}>
          {isEditing ? (
            <>
              <Button variant="outline" onClick={handleCancel}>
                Cancel
              </Button>
              <Button colorScheme="blue" onClick={handleSave}>
                Save Changes
              </Button>
            </>
          ) : (
            <Button colorScheme="blue" onClick={() => setIsEditing(true)}>
              Edit Profile
            </Button>
          )}
        </HStack>
      </HStack>

      {/* Profile Information */}
      <SimpleGrid columns={{ base: 1, lg: 2 }} gap={6}>
        {/* Basic Information */}
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          <VStack align="stretch" gap={4}>
            <HStack justify="space-between">
              <Heading size="md">Basic Information</Heading>
              <Badge colorScheme="green" px={3} py={1} borderRadius="full">
                Active
              </Badge>
            </HStack>
            
            <SimpleGrid columns={2} gap={4}>
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  First Name
                </Text>
                {isEditing ? (
                  <Input
                    value={formData.firstName}
                    onChange={(e) => handleInputChange('firstName', e.target.value)}
                    size="sm"
                  />
                ) : (
                  <Text fontSize="sm">{formData.firstName}</Text>
                )}
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Last Name
                </Text>
                {isEditing ? (
                  <Input
                    value={formData.lastName}
                    onChange={(e) => handleInputChange('lastName', e.target.value)}
                    size="sm"
                  />
                ) : (
                  <Text fontSize="sm">{formData.lastName}</Text>
                )}
              </Box>
            </SimpleGrid>

            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Username
              </Text>
              {isEditing ? (
                <Input
                  value={formData.username}
                  onChange={(e) => handleInputChange('username', e.target.value)}
                  size="sm"
                />
              ) : (
                <Text fontSize="sm">{formData.username}</Text>
              )}
            </Box>

            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Email Address
              </Text>
              {isEditing ? (
                <Input
                  value={formData.email}
                  onChange={(e) => handleInputChange('email', e.target.value)}
                  size="sm"
                  type="email"
                />
              ) : (
                <Text fontSize="sm">{formData.email}</Text>
              )}
            </Box>

            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Phone Number
              </Text>
              {isEditing ? (
                <Input
                  value={formData.phone}
                  onChange={(e) => handleInputChange('phone', e.target.value)}
                  size="sm"
                />
              ) : (
                <Text fontSize="sm">{formData.phone}</Text>
              )}
            </Box>
          </VStack>
        </Box>

        {/* Role & Access Information */}
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          <VStack align="stretch" gap={4}>
            <Heading size="md">Role & Access</Heading>
            
            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Role
              </Text>
              <HStack>
                <Badge colorScheme="purple" px={3} py={1} borderRadius="full">
                  {formData.role}
                </Badge>
              </HStack>
            </Box>

            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Department
              </Text>
              {isEditing ? (
                <Input
                  value={formData.department}
                  onChange={(e) => handleInputChange('department', e.target.value)}
                  size="sm"
                />
              ) : (
                <Text fontSize="sm">{formData.department}</Text>
              )}
            </Box>

            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Location
              </Text>
              {isEditing ? (
                <Input
                  value={formData.location}
                  onChange={(e) => handleInputChange('location', e.target.value)}
                  size="sm"
                />
              ) : (
                <Text fontSize="sm">{formData.location}</Text>
              )}
            </Box>

            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Last Login
              </Text>
              <Text fontSize="sm" color="gray.600">
                Today at 9:32 AM
              </Text>
            </Box>

            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Account Created
              </Text>
              <Text fontSize="sm" color="gray.600">
                January 15, 2024
              </Text>
            </Box>
          </VStack>
        </Box>
      </SimpleGrid>

      {/* Account Activity */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <Heading size="md">Recent Account Activity</Heading>
          <VStack align="stretch" gap={2}>
            <Box p={3} bg="gray.50" borderRadius="md">
              <HStack justify="space-between">
                <VStack align="start" gap={0}>
                  <Text fontSize="sm" fontWeight="medium">Login from Chrome on Windows</Text>
                  <Text fontSize="xs" color="gray.500">IP: 192.168.1.100</Text>
                </VStack>
                <Text fontSize="xs" color="gray.500">Today, 9:32 AM</Text>
              </HStack>
            </Box>
            
            <Box p={3} bg="gray.50" borderRadius="md">
              <HStack justify="space-between">
                <VStack align="start" gap={0}>
                  <Text fontSize="sm" fontWeight="medium">Profile updated</Text>
                  <Text fontSize="xs" color="gray.500">Phone number changed</Text>
                </VStack>
                <Text fontSize="xs" color="gray.500">Yesterday, 3:15 PM</Text>
              </HStack>
            </Box>
            
            <Box p={3} bg="gray.50" borderRadius="md">
              <HStack justify="space-between">
                <VStack align="start" gap={0}>
                  <Text fontSize="sm" fontWeight="medium">Password changed</Text>
                  <Text fontSize="xs" color="gray.500">Security settings updated</Text>
                </VStack>
                <Text fontSize="xs" color="gray.500">3 days ago</Text>
              </HStack>
            </Box>
          </VStack>
        </VStack>
      </Box>
    </VStack>
  );
};

export default UserProfile;