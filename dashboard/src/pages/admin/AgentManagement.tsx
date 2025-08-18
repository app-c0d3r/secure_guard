import React, { useState } from 'react';
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Button,
  Badge,
  SimpleGrid,
  Input,
  Textarea,
} from '@chakra-ui/react';

interface AgentVersion {
  id: string;
  version: string;
  platform: 'Windows' | 'Linux' | 'macOS';
  architecture: 'x64' | 'x86' | 'ARM64';
  fileSize: string;
  releaseDate: string;
  status: 'stable' | 'beta' | 'deprecated';
  downloadCount: number;
  changelog: string;
  checksum: string;
}

const AgentManagement: React.FC = () => {
  const [showUpload, setShowUpload] = useState(false);
  const [uploadData, setUploadData] = useState({
    version: '',
    platform: 'Windows' as const,
    architecture: 'x64' as const,
    changelog: '',
  });

  // Mock data - would come from API
  const [agentVersions] = useState<AgentVersion[]>([
    {
      id: '1',
      version: '2.4.1',
      platform: 'Windows',
      architecture: 'x64',
      fileSize: '45.2 MB',
      releaseDate: '2024-01-20',
      status: 'stable',
      downloadCount: 1247,
      changelog: 'Bug fixes, performance improvements, new threat detection algorithms',
      checksum: 'sha256:a1b2c3d4e5f6...',
    },
    {
      id: '2',
      version: '2.4.1',
      platform: 'Linux',
      architecture: 'x64',
      fileSize: '38.7 MB',
      releaseDate: '2024-01-20',
      status: 'stable',
      downloadCount: 892,
      changelog: 'Bug fixes, performance improvements, new threat detection algorithms',
      checksum: 'sha256:f6e5d4c3b2a1...',
    },
    {
      id: '3',
      version: '2.4.0',
      platform: 'Windows',
      architecture: 'x64',
      fileSize: '44.8 MB',
      releaseDate: '2024-01-15',
      status: 'deprecated',
      downloadCount: 2156,
      changelog: 'Major feature updates, UI improvements, enhanced reporting',
      checksum: 'sha256:b2c3d4e5f6a1...',
    },
    {
      id: '4',
      version: '2.5.0-beta',
      platform: 'Windows',
      architecture: 'x64',
      fileSize: '46.1 MB',
      releaseDate: '2024-01-22',
      status: 'beta',
      downloadCount: 156,
      changelog: 'Beta release with AI-powered threat analysis (experimental)',
      checksum: 'sha256:c3d4e5f6a1b2...',
    },
  ]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'stable': return 'green';
      case 'beta': return 'orange';
      case 'deprecated': return 'red';
      default: return 'gray';
    }
  };

  const getPlatformColor = (platform: string) => {
    switch (platform) {
      case 'Windows': return 'blue';
      case 'Linux': return 'green';
      case 'macOS': return 'purple';
      default: return 'gray';
    }
  };

  const handleUpload = () => {
    // Here you would typically handle file upload to the server
    console.log('Uploading agent:', uploadData);
    setShowUpload(false);
    setUploadData({
      version: '',
      platform: 'Windows',
      architecture: 'x64',
      changelog: '',
    });
  };

  const handleDownload = (agent: AgentVersion) => {
    // Here you would trigger the download
    console.log('Downloading agent:', agent);
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Box>
          <Heading size="lg">Agent Management</Heading>
          <Text color="gray.600" mt={1}>
            Manage SecureGuard agent versions and downloads
          </Text>
        </Box>
        <Button colorScheme="blue" onClick={() => setShowUpload(true)}>
          Upload New Version
        </Button>
      </HStack>

      {/* Statistics */}
      <SimpleGrid columns={{ base: 2, md: 4 }} gap={4}>
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="blue.600">
            {agentVersions.filter(a => a.status === 'stable').length}
          </Text>
          <Text fontSize="sm" color="gray.600">Stable Versions</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="orange.600">
            {agentVersions.filter(a => a.status === 'beta').length}
          </Text>
          <Text fontSize="sm" color="gray.600">Beta Versions</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="green.600">
            {agentVersions.reduce((sum, agent) => sum + agent.downloadCount, 0)}
          </Text>
          <Text fontSize="sm" color="gray.600">Total Downloads</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="purple.600">
            {new Set(agentVersions.map(a => a.platform)).size}
          </Text>
          <Text fontSize="sm" color="gray.600">Platforms</Text>
        </Box>
      </SimpleGrid>

      {/* Upload Form */}
      {showUpload && (
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          <VStack align="stretch" gap={4}>
            <HStack justify="space-between">
              <Heading size="md">Upload New Agent Version</Heading>
              <Button variant="outline" onClick={() => setShowUpload(false)}>
                Cancel
              </Button>
            </HStack>
            
            <SimpleGrid columns={{ base: 1, md: 2 }} gap={4}>
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Version Number
                </Text>
                <Input
                  placeholder="e.g., 2.4.2"
                  value={uploadData.version}
                  onChange={(e) => setUploadData({...uploadData, version: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Platform
                </Text>
                <HStack gap={2}>
                  {['Windows', 'Linux', 'macOS'].map(platform => (
                    <Button
                      key={platform}
                      size="sm"
                      variant={uploadData.platform === platform ? 'solid' : 'outline'}
                      colorScheme={uploadData.platform === platform ? 'blue' : 'gray'}
                      onClick={() => setUploadData({...uploadData, platform: platform as any})}
                    >
                      {platform}
                    </Button>
                  ))}
                </HStack>
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Architecture
                </Text>
                <HStack gap={2}>
                  {['x64', 'x86', 'ARM64'].map(arch => (
                    <Button
                      key={arch}
                      size="sm"
                      variant={uploadData.architecture === arch ? 'solid' : 'outline'}
                      colorScheme={uploadData.architecture === arch ? 'blue' : 'gray'}
                      onClick={() => setUploadData({...uploadData, architecture: arch as any})}
                    >
                      {arch}
                    </Button>
                  ))}
                </HStack>
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Agent File
                </Text>
                <Box
                  p={8}
                  border="2px dashed"
                  borderColor="gray.300"
                  borderRadius="md"
                  textAlign="center"
                  _hover={{ borderColor: 'blue.400', bg: 'blue.50' }}
                  cursor="pointer"
                >
                  <Text fontSize="sm" color="gray.600">
                    Click to upload or drag and drop
                  </Text>
                  <Text fontSize="xs" color="gray.500">
                    Supported formats: .exe, .msi, .deb, .rpm, .dmg
                  </Text>
                </Box>
              </Box>
            </SimpleGrid>
            
            <Box>
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Changelog
              </Text>
              <Textarea
                placeholder="Describe the changes in this version..."
                value={uploadData.changelog}
                onChange={(e) => setUploadData({...uploadData, changelog: e.target.value})}
                size="sm"
                resize="vertical"
                rows={4}
              />
            </Box>
            
            <HStack justify="end">
              <Button colorScheme="blue" onClick={handleUpload}>
                Upload Agent
              </Button>
            </HStack>
          </VStack>
        </Box>
      )}

      {/* Agent Versions Table */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <Heading size="md">Available Agent Versions</Heading>
          
          {/* Table Header */}
          <Box bg="gray.50" p={3} borderRadius="md">
            <HStack>
              <Text fontWeight="medium" fontSize="sm" flex="2">Version & Platform</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Status</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">File Size</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Downloads</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Release Date</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Actions</Text>
            </HStack>
          </Box>
          
          {/* Table Body */}
          <VStack gap={0}>
            {agentVersions.map((agent, index) => (
              <Box key={agent.id} w="100%">
                <HStack p={3} _hover={{ bg: 'gray.50' }}>
                  <Box flex="2">
                    <VStack align="start" gap={1}>
                      <HStack>
                        <Text fontWeight="medium" fontSize="sm">
                          SecureGuard Agent v{agent.version}
                        </Text>
                        <Badge
                          colorScheme={getPlatformColor(agent.platform)}
                          variant="subtle"
                          fontSize="xs"
                        >
                          {agent.platform} {agent.architecture}
                        </Badge>
                      </HStack>
                      <Text 
                        fontSize="xs" 
                        color="gray.500"
                        overflow="hidden"
                        textOverflow="ellipsis"
                        whiteSpace="nowrap"
                      >
                        {agent.changelog}
                      </Text>
                    </VStack>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getStatusColor(agent.status)}
                      variant="subtle"
                      fontSize="xs"
                      textTransform="capitalize"
                    >
                      {agent.status}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Text fontSize="sm">{agent.fileSize}</Text>
                  </Box>
                  
                  <Box flex="1">
                    <Text fontSize="sm" color="gray.600">
                      {agent.downloadCount.toLocaleString()}
                    </Text>
                  </Box>
                  
                  <Box flex="1">
                    <Text fontSize="sm" color="gray.600">
                      {agent.releaseDate}
                    </Text>
                  </Box>
                  
                  <Box flex="1">
                    <HStack gap={2}>
                      <Button 
                        size="xs" 
                        colorScheme="blue" 
                        onClick={() => handleDownload(agent)}
                      >
                        Download
                      </Button>
                      <Button size="xs" variant="outline" colorScheme="gray">
                        Details
                      </Button>
                    </HStack>
                  </Box>
                </HStack>
                {index < agentVersions.length - 1 && <Box h="1px" bg="gray.100" />}
              </Box>
            ))}
          </VStack>
        </VStack>
      </Box>

      {/* Download Instructions */}
      <Box bg="blue.50" p={6} borderRadius="md" border="1px" borderColor="blue.200">
        <VStack align="stretch" gap={3}>
          <Heading size="sm" color="blue.700">
            Agent Installation Instructions
          </Heading>
          <Text fontSize="sm" color="blue.600">
            1. Download the appropriate version for your operating system and architecture
          </Text>
          <Text fontSize="sm" color="blue.600">
            2. Verify the checksum to ensure file integrity
          </Text>
          <Text fontSize="sm" color="blue.600">
            3. Run the installer with administrator privileges
          </Text>
          <Text fontSize="sm" color="blue.600">
            4. Follow the setup wizard and configure connection to SecureGuard platform
          </Text>
          <Text fontSize="sm" color="blue.600">
            5. The agent will automatically register and begin monitoring
          </Text>
        </VStack>
      </Box>
    </VStack>
  );
};

export default AgentManagement;