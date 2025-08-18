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
import { FiDownload, FiBarChart, FiFileText, FiCalendar } from 'react-icons/fi';

const Reports: React.FC = () => {
  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Heading size="lg">Security Reports</Heading>
        <Button colorScheme="blue">
          Export All
        </Button>
      </HStack>

      {/* Report Cards */}
      <SimpleGrid columns={{ base: 1, md: 2, lg: 3 }} gap={6}>
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
          <VStack align="start" gap={4}>
            <HStack>
              <Box w={6} h={6} bg="blue.500" borderRadius="md" />
              <Heading size="md">Threat Analysis Report</Heading>
            </HStack>
            <Text color="gray.600" fontSize="sm">
              Comprehensive analysis of detected threats, attack patterns, and security incidents over the past 30 days.
            </Text>
            <Button size="sm" variant="outline">
              Generate Report
            </Button>
          </VStack>
        </Box>

        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
          <VStack align="start" gap={4}>
            <HStack>
              <Box w={6} h={6} bg="green.500" borderRadius="md" />
              <Heading size="md">Agent Performance</Heading>
            </HStack>
            <Text color="gray.600" fontSize="sm">
              Detailed performance metrics for all connected agents including uptime, response times, and health status.
            </Text>
            <Button size="sm" variant="outline">
              Generate Report
            </Button>
          </VStack>
        </Box>

        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm">
          <VStack align="start" gap={4}>
            <HStack>
              <Box w={6} h={6} bg="purple.500" borderRadius="md" />
              <Heading size="md">Compliance Report</Heading>
            </HStack>
            <Text color="gray.600" fontSize="sm">
              Security compliance status and audit trail for regulatory requirements and internal policies.
            </Text>
            <Button size="sm" variant="outline">
              Generate Report
            </Button>
          </VStack>
        </Box>
      </SimpleGrid>

      {/* Coming Soon */}
      <Box bg="white" p={12} borderRadius="md" border="1px" borderColor="gray.200" shadow="sm" textAlign="center">
        <Box w={16} h={16} bg="gray.300" borderRadius="md" mx="auto" mb={4} />
        <Heading size="lg" color="gray.500" mb={2}>
          Advanced Reporting Coming Soon
        </Heading>
        <Text color="gray.400">
          Detailed analytics, custom reports, and automated scheduling will be available in the next release.
        </Text>
      </Box>
    </VStack>
  );
};

export default Reports;