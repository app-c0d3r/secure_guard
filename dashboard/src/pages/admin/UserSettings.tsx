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

const UserSettings: React.FC = () => {
  const [activeSection, setActiveSection] = useState('password');
  const [passwordData, setPasswordData] = useState({
    currentPassword: '',
    newPassword: '',
    confirmPassword: '',
  });
  const [accountData, setAccountData] = useState({
    username: 'admin.user',
    email: 'admin@secureguard.com',
    newUsername: '',
    newEmail: '',
  });
  const [securitySettings, setSecuritySettings] = useState({
    twoFactorEnabled: true,
    sessionTimeout: 30,
    passwordExpiry: 90,
  });

  const handlePasswordChange = () => {
    if (passwordData.newPassword !== passwordData.confirmPassword) {
      alert('Passwords do not match');
      return;
    }
    console.log('Changing password:', passwordData);
    setPasswordData({
      currentPassword: '',
      newPassword: '',
      confirmPassword: '',
    });
  };

  const handleUsernameChange = () => {
    console.log('Changing username from', accountData.username, 'to', accountData.newUsername);
    setAccountData(prev => ({
      ...prev,
      username: prev.newUsername,
      newUsername: '',
    }));
  };

  const handleEmailChange = () => {
    console.log('Changing email from', accountData.email, 'to', accountData.newEmail);
    setAccountData(prev => ({
      ...prev,
      email: prev.newEmail,
      newEmail: '',
    }));
  };

  const toggleTwoFactor = () => {
    setSecuritySettings(prev => ({
      ...prev,
      twoFactorEnabled: !prev.twoFactorEnabled,
    }));
  };

  const menuItems = [
    { id: 'password', label: 'Password & Security', icon: '🔐' },
    { id: 'account', label: 'Account Information', icon: '👤' },
    { id: 'security', label: 'Security Settings', icon: '🛡️' },
    { id: 'sessions', label: 'Active Sessions', icon: '📱' },
    { id: 'notifications', label: 'Notifications', icon: '🔔' },
  ];

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <Box>
        <Heading size="lg">User Settings</Heading>
        <Text color="gray.600" mt={1}>
          Manage your account settings, security preferences, and personal information
        </Text>
      </Box>

      <HStack align="start" gap={6}>
        {/* Side Navigation */}
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200" minW="250px">
          <VStack gap={2} align="stretch">
            {menuItems.map(item => (
              <Button
                key={item.id}
                variant={activeSection === item.id ? 'solid' : 'ghost'}
                colorScheme={activeSection === item.id ? 'blue' : 'gray'}
                justifyContent="flex-start"
                size="sm"
                onClick={() => setActiveSection(item.id)}
              >
                <HStack>
                  <Text>{item.icon}</Text>
                  <Text>{item.label}</Text>
                </HStack>
              </Button>
            ))}
          </VStack>
        </Box>

        {/* Main Content */}
        <Box flex="1" bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          {/* Password & Security Section */}
          {activeSection === 'password' && (
            <VStack align="stretch" gap={6}>
              <Box>
                <Heading size="md" mb={4}>Password & Security</Heading>
                
                {/* Change Password */}
                <Box mb={6}>
                  <Heading size="sm" mb={3}>Change Password</Heading>
                  <VStack gap={3} align="stretch">
                    <Box>
                      <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                        Current Password
                      </Text>
                      <Input
                        type="password"
                        value={passwordData.currentPassword}
                        onChange={(e) => setPasswordData({...passwordData, currentPassword: e.target.value})}
                        size="sm"
                        placeholder="Enter current password"
                      />
                    </Box>
                    
                    <Box>
                      <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                        New Password
                      </Text>
                      <Input
                        type="password"
                        value={passwordData.newPassword}
                        onChange={(e) => setPasswordData({...passwordData, newPassword: e.target.value})}
                        size="sm"
                        placeholder="Enter new password"
                      />
                      <Text fontSize="xs" color="gray.500" mt={1}>
                        Must be at least 8 characters with uppercase, lowercase, number, and special character
                      </Text>
                    </Box>
                    
                    <Box>
                      <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                        Confirm New Password
                      </Text>
                      <Input
                        type="password"
                        value={passwordData.confirmPassword}
                        onChange={(e) => setPasswordData({...passwordData, confirmPassword: e.target.value})}
                        size="sm"
                        placeholder="Confirm new password"
                      />
                    </Box>
                    
                    <HStack>
                      <Button colorScheme="blue" size="sm" onClick={handlePasswordChange}>
                        Change Password
                      </Button>
                      <Button variant="outline" size="sm">
                        Cancel
                      </Button>
                    </HStack>
                  </VStack>
                </Box>

                {/* Password Requirements */}
                <Box bg="blue.50" p={4} borderRadius="md" border="1px" borderColor="blue.200">
                  <Text fontSize="sm" fontWeight="medium" color="blue.700" mb={2}>
                    Password Requirements
                  </Text>
                  <VStack align="start" gap={1}>
                    <Text fontSize="xs" color="blue.600">• Minimum 8 characters</Text>
                    <Text fontSize="xs" color="blue.600">• At least one uppercase letter</Text>
                    <Text fontSize="xs" color="blue.600">• At least one lowercase letter</Text>
                    <Text fontSize="xs" color="blue.600">• At least one number</Text>
                    <Text fontSize="xs" color="blue.600">• At least one special character</Text>
                    <Text fontSize="xs" color="blue.600">• Cannot be a previously used password</Text>
                  </VStack>
                </Box>
              </Box>
            </VStack>
          )}

          {/* Account Information Section */}
          {activeSection === 'account' && (
            <VStack align="stretch" gap={6}>
              <Heading size="md">Account Information</Heading>
              
              {/* Username Change */}
              <Box>
                <Heading size="sm" mb={3}>Username</Heading>
                <VStack gap={3} align="stretch">
                  <Box>
                    <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                      Current Username
                    </Text>
                    <Input
                      value={accountData.username}
                      readOnly
                      bg="gray.50"
                      size="sm"
                    />
                  </Box>
                  
                  <Box>
                    <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                      New Username
                    </Text>
                    <Input
                      value={accountData.newUsername}
                      onChange={(e) => setAccountData({...accountData, newUsername: e.target.value})}
                      size="sm"
                      placeholder="Enter new username"
                    />
                    <Text fontSize="xs" color="gray.500" mt={1}>
                      Username must be unique and contain only letters, numbers, and underscores
                    </Text>
                  </Box>
                  
                  <HStack>
                    <Button colorScheme="blue" size="sm" onClick={handleUsernameChange}>
                      Change Username
                    </Button>
                    <Button variant="outline" size="sm">
                      Cancel
                    </Button>
                  </HStack>
                </VStack>
              </Box>

              {/* Email Change */}
              <Box>
                <Heading size="sm" mb={3}>Email Address</Heading>
                <VStack gap={3} align="stretch">
                  <Box>
                    <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                      Current Email
                    </Text>
                    <Input
                      value={accountData.email}
                      readOnly
                      bg="gray.50"
                      size="sm"
                    />
                  </Box>
                  
                  <Box>
                    <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                      New Email Address
                    </Text>
                    <Input
                      type="email"
                      value={accountData.newEmail}
                      onChange={(e) => setAccountData({...accountData, newEmail: e.target.value})}
                      size="sm"
                      placeholder="Enter new email address"
                    />
                    <Text fontSize="xs" color="gray.500" mt={1}>
                      A verification email will be sent to the new address
                    </Text>
                  </Box>
                  
                  <HStack>
                    <Button colorScheme="blue" size="sm" onClick={handleEmailChange}>
                      Change Email
                    </Button>
                    <Button variant="outline" size="sm">
                      Cancel
                    </Button>
                  </HStack>
                </VStack>
              </Box>
            </VStack>
          )}

          {/* Security Settings Section */}
          {activeSection === 'security' && (
            <VStack align="stretch" gap={6}>
              <Heading size="md">Security Settings</Heading>
              
              {/* Two-Factor Authentication */}
              <Box>
                <HStack justify="space-between" mb={3}>
                  <Box>
                    <Heading size="sm">Two-Factor Authentication</Heading>
                    <Text fontSize="sm" color="gray.600">
                      Add an extra layer of security to your account
                    </Text>
                  </Box>
                  <Badge colorScheme={securitySettings.twoFactorEnabled ? 'green' : 'red'}>
                    {securitySettings.twoFactorEnabled ? 'Enabled' : 'Disabled'}
                  </Badge>
                </HStack>
                
                <Button
                  colorScheme={securitySettings.twoFactorEnabled ? 'red' : 'green'}
                  size="sm"
                  onClick={toggleTwoFactor}
                >
                  {securitySettings.twoFactorEnabled ? 'Disable 2FA' : 'Enable 2FA'}
                </Button>
              </Box>

              {/* Session Settings */}
              <Box>
                <Heading size="sm" mb={3}>Session Settings</Heading>
                <SimpleGrid columns={2} gap={4}>
                  <Box>
                    <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                      Session Timeout (minutes)
                    </Text>
                    <Input
                      type="number"
                      value={securitySettings.sessionTimeout}
                      onChange={(e) => setSecuritySettings({
                        ...securitySettings,
                        sessionTimeout: parseInt(e.target.value) || 30
                      })}
                      size="sm"
                    />
                  </Box>
                  
                  <Box>
                    <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                      Password Expiry (days)
                    </Text>
                    <Input
                      type="number"
                      value={securitySettings.passwordExpiry}
                      onChange={(e) => setSecuritySettings({
                        ...securitySettings,
                        passwordExpiry: parseInt(e.target.value) || 90
                      })}
                      size="sm"
                    />
                  </Box>
                </SimpleGrid>
              </Box>

              {/* Security Actions */}
              <Box bg="orange.50" p={4} borderRadius="md" border="1px" borderColor="orange.200">
                <Heading size="sm" color="orange.700" mb={3}>Security Actions</Heading>
                <VStack align="stretch" gap={2}>
                  <Button size="sm" colorScheme="orange" variant="outline">
                    Force Logout All Devices
                  </Button>
                  <Button size="sm" colorScheme="red" variant="outline">
                    Reset Security Settings
                  </Button>
                </VStack>
              </Box>
            </VStack>
          )}

          {/* Active Sessions Section */}
          {activeSection === 'sessions' && (
            <VStack align="stretch" gap={6}>
              <Heading size="md">Active Sessions</Heading>
              
              <VStack gap={3} align="stretch">
                {/* Current Session */}
                <Box p={4} bg="green.50" borderRadius="md" border="1px" borderColor="green.200">
                  <HStack justify="space-between">
                    <VStack align="start" gap={1}>
                      <Text fontSize="sm" fontWeight="medium">Current Session</Text>
                      <Text fontSize="xs" color="gray.600">Chrome on Windows • 192.168.1.100</Text>
                      <Text fontSize="xs" color="gray.500">Started: Today at 9:32 AM</Text>
                    </VStack>
                    <Badge colorScheme="green">Active</Badge>
                  </HStack>
                </Box>

                {/* Other Sessions */}
                <Box p={4} bg="gray.50" borderRadius="md" border="1px" borderColor="gray.200">
                  <HStack justify="space-between">
                    <VStack align="start" gap={1}>
                      <Text fontSize="sm" fontWeight="medium">Mobile Session</Text>
                      <Text fontSize="xs" color="gray.600">Safari on iOS • 192.168.1.105</Text>
                      <Text fontSize="xs" color="gray.500">Last active: Yesterday at 6:15 PM</Text>
                    </VStack>
                    <Button size="xs" colorScheme="red" variant="outline">
                      Terminate
                    </Button>
                  </HStack>
                </Box>

                <Box p={4} bg="gray.50" borderRadius="md" border="1px" borderColor="gray.200">
                  <HStack justify="space-between">
                    <VStack align="start" gap={1}>
                      <Text fontSize="sm" fontWeight="medium">Laptop Session</Text>
                      <Text fontSize="xs" color="gray.600">Firefox on macOS • 192.168.1.102</Text>
                      <Text fontSize="xs" color="gray.500">Last active: 3 hours ago</Text>
                    </VStack>
                    <Button size="xs" colorScheme="red" variant="outline">
                      Terminate
                    </Button>
                  </HStack>
                </Box>
              </VStack>

              <Button colorScheme="red" size="sm" alignSelf="start">
                Terminate All Other Sessions
              </Button>
            </VStack>
          )}

          {/* Notifications Section */}
          {activeSection === 'notifications' && (
            <VStack align="stretch" gap={6}>
              <Heading size="md">Notification Preferences</Heading>
              
              <VStack gap={4} align="stretch">
                {/* Email Notifications */}
                <Box>
                  <Heading size="sm" mb={3}>Email Notifications</Heading>
                  <VStack gap={3} align="stretch">
                    <HStack justify="space-between">
                      <VStack align="start" gap={0}>
                        <Text fontSize="sm" fontWeight="medium">Security Alerts</Text>
                        <Text fontSize="xs" color="gray.500">Critical security events and threats</Text>
                      </VStack>
                      <Button size="xs" colorScheme="green">Enabled</Button>
                    </HStack>
                    
                    <HStack justify="space-between">
                      <VStack align="start" gap={0}>
                        <Text fontSize="sm" fontWeight="medium">System Updates</Text>
                        <Text fontSize="xs" color="gray.500">Platform updates and maintenance</Text>
                      </VStack>
                      <Button size="xs" variant="outline">Disabled</Button>
                    </HStack>
                    
                    <HStack justify="space-between">
                      <VStack align="start" gap={0}>
                        <Text fontSize="sm" fontWeight="medium">Weekly Reports</Text>
                        <Text fontSize="xs" color="gray.500">Weekly security summary reports</Text>
                      </VStack>
                      <Button size="xs" colorScheme="green">Enabled</Button>
                    </HStack>
                  </VStack>
                </Box>

                {/* In-App Notifications */}
                <Box>
                  <Heading size="sm" mb={3}>In-App Notifications</Heading>
                  <VStack gap={3} align="stretch">
                    <HStack justify="space-between">
                      <VStack align="start" gap={0}>
                        <Text fontSize="sm" fontWeight="medium">Real-time Alerts</Text>
                        <Text fontSize="xs" color="gray.500">Immediate threat notifications</Text>
                      </VStack>
                      <Button size="xs" colorScheme="green">Enabled</Button>
                    </HStack>
                    
                    <HStack justify="space-between">
                      <VStack align="start" gap={0}>
                        <Text fontSize="sm" fontWeight="medium">Task Reminders</Text>
                        <Text fontSize="xs" color="gray.500">Reminders for pending tasks</Text>
                      </VStack>
                      <Button size="xs" colorScheme="green">Enabled</Button>
                    </HStack>
                  </VStack>
                </Box>
              </VStack>
            </VStack>
          )}
        </Box>
      </HStack>
    </VStack>
  );
};

export default UserSettings;