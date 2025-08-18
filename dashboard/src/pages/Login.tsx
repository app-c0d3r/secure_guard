import React, { useState } from 'react';
import {
  Box,
  VStack,
  HStack,
  Text,
  Input,
  Button,
  Heading,
  Flex,
} from '@chakra-ui/react';
import { FiEye, FiEyeOff, FiShield, FiLock, FiUser } from 'react-icons/fi';
import { useAuth } from '../contexts/AuthContext';

const Login: React.FC = () => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const { login } = useAuth();
  const bg = 'gray.50';
  const cardBg = 'white';

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!username.trim() || !password.trim()) {
      setError('Please enter both username and password');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const success = await login(username.trim(), password);
      if (!success) {
        setError('Invalid username or password');
      }
    } catch (err) {
      setError('Login failed. Please try again.');
      console.error('Login error:', err);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Flex minH="100vh" align="center" justify="center" bg={bg}>
      <Box w="full" maxW="md" px={6}>
        <Box bg={cardBg} shadow="xl" borderRadius="xl" border="1px" borderColor="gray.200" p={8}>
          <VStack gap={6}>
            {/* Logo and Title */}
            <VStack gap={4}>
              <Box
                p={4}
                borderRadius="full"
                bg="blue.500"
                color="white"
              >
                <Box w={8} h={8} bg="white" borderRadius="md" />
              </Box>
              <VStack gap={1}>
                <Heading size="lg" color="blue.500" textAlign="center">
                  SecureGuard
                </Heading>
                <Text color="gray.600" textAlign="center" fontSize="sm">
                  Cybersecurity Management Platform
                </Text>
              </VStack>
            </VStack>

            {/* Login Form */}
            <Box w="full">
              <form onSubmit={handleSubmit}>
                <VStack gap={4}>
                  {error && (
                    <Box p={3} bg="red.50" border="1px" borderColor="red.200" borderRadius="md" w="full">
                      <Text fontSize="sm" color="red.600">
                        {error}
                      </Text>
                    </Box>
                  )}

                  <Box w="full">
                    <Text mb={2} fontSize="sm" fontWeight="medium" color="gray.700">
                      Username
                    </Text>
                    <Input
                      type="text"
                      value={username}
                      onChange={(e) => setUsername(e.target.value)}
                      placeholder="Enter your username"
                      size="lg"
                      bg="gray.50"
                      border="1px"
                      borderColor="gray.200"
                      _focus={{
                        borderColor: 'blue.500',
                        bg: 'white',
                        boxShadow: '0 0 0 1px #4299E1',
                      }}
                    />
                  </Box>

                  <Box w="full">
                    <Text mb={2} fontSize="sm" fontWeight="medium" color="gray.700">
                      Password
                    </Text>
                    <Box position="relative">
                      <Input
                        type={showPassword ? 'text' : 'password'}
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        placeholder="Enter your password"
                        size="lg"
                        bg="gray.50"
                        border="1px"
                        borderColor="gray.200"
                        pr="3rem"
                        _focus={{
                          borderColor: 'blue.500',
                          bg: 'white',
                          boxShadow: '0 0 0 1px #4299E1',
                        }}
                      />
                      <button
                        type="button"
                        style={{
                          position: 'absolute',
                          right: '12px',
                          top: '50%',
                          transform: 'translateY(-50%)',
                          padding: '4px',
                          background: 'transparent',
                          border: 'none',
                          borderRadius: '4px',
                          cursor: 'pointer',
                        }}
                        onClick={() => setShowPassword(!showPassword)}
                        onMouseOver={(e) => (e.currentTarget.style.backgroundColor = '#f7fafc')}
                        onMouseOut={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
                      >
                        <Box w={4} h={4} bg="gray.500" borderRadius="sm" />
                      </button>
                    </Box>
                  </Box>

                  <Button
                    type="submit"
                    w="full"
                    size="lg"
                    colorScheme="blue"
                    disabled={isLoading}
                  >
                    {isLoading ? 'Signing In...' : 'Sign In'}
                  </Button>
                </VStack>
              </form>
            </Box>

            {/* Demo Credentials */}
            <Box w="full" p={4} bg="blue.50" borderRadius="md">
              <VStack gap={2}>
                <Text fontSize="sm" fontWeight="semibold" color="blue.700">
                  Demo Credentials
                </Text>
                <VStack gap={1}>
                  <HStack gap={2} fontSize="xs">
                    <Box w={3} h={3} bg="blue.600" borderRadius="sm" />
                    <Text color="blue.600">
                      <strong>Username:</strong> admin
                    </Text>
                  </HStack>
                  <HStack gap={2} fontSize="xs">
                    <Box w={3} h={3} bg="blue.600" borderRadius="sm" />
                    <Text color="blue.600">
                      <strong>Password:</strong> admin123
                    </Text>
                  </HStack>
                </VStack>
              </VStack>
            </Box>

            {/* Footer */}
            <Text fontSize="xs" color="gray.500" textAlign="center">
              Powered by SecureGuard Cybersecurity Platform
              <br />
              © 2024 - Advanced Threat Detection & Response
            </Text>
          </VStack>
        </Box>
      </Box>
    </Flex>
  );
};

export default Login;