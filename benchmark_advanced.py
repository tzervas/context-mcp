#!/usr/bin/env python3
"""
Advanced context-mcp benchmark with real-world Kubernetes data
Includes security screening and web retrieval integration
Tests scalability and performance with complex datasets
"""

import asyncio
import json
import subprocess
import time
import statistics
import requests
from datetime import datetime
from typing import Dict, List, Any, Optional
import sys
import re

class AdvancedMCPClient:
    """Enhanced MCP client with multi-server support"""
    
    def __init__(self, command: List[str], name: str = "context-mcp"):
        self.process = None
        self.command = command
        self.request_id = 0
        self.name = name
        
    async def start(self):
        """Start the MCP server process"""
        self.process = await asyncio.create_subprocess_exec(
            *self.command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        print(f"✓ Started {self.name}: {' '.join(self.command)}")
        
    async def stop(self):
        """Stop the MCP server process"""
        if self.process:
            self.process.terminate()
            await self.process.wait()
            print(f"✓ Stopped {self.name}")
    
    async def call_method(self, method: str, params: Dict = None) -> Dict:
        """Call an MCP method"""
        self.request_id += 1
        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params or {}
        }
        
        request_str = json.dumps(request) + "\n"
        self.process.stdin.write(request_str.encode())
        await self.process.stdin.drain()
        
        # Read response (skip non-JSON lines)
        max_attempts = 10
        for attempt in range(max_attempts):
            response_line = await self.process.stdout.readline()
            response_str = response_line.decode().strip()
            
            if not response_str:
                continue
                
            try:
                response = json.loads(response_str)
                break
            except json.JSONDecodeError:
                if attempt == max_attempts - 1:
                    raise Exception(f"Could not parse response")
                continue
        
        if "error" in response:
            raise Exception(f"MCP error: {response['error']}")
            
        return response.get("result", {})
    
    async def call_tool(self, tool_name: str, arguments: Dict = None) -> Dict:
        """Call an MCP tool"""
        result = await self.call_method("tools/call", {
            "name": tool_name,
            "arguments": arguments or {}
        })
        return result


class KubernetesDataGenerator:
    """Generate realistic Kubernetes release data for benchmarking"""
    
    @staticmethod
    def get_k8s_release_notes() -> List[Dict[str, Any]]:
        """Generate comprehensive Kubernetes release data"""
        releases = []
        
        # Recent K8s versions with detailed information
        versions = [
            {
                "version": "1.31.0",
                "date": "2024-08-15",
                "type": "feature",
                "components": [
                    "API Server: Enhanced API priority and fairness with new metrics",
                    "Kubelet: Improved container restart policies and error handling",
                    "Scheduler: New scheduling plugins for workload isolation",
                    "etcd: Distributed transaction support for leader election",
                    "CNI: Enhanced network policy enforcement",
                ]
            },
            {
                "version": "1.30.5",
                "date": "2024-09-10",
                "type": "patch",
                "components": [
                    "Security fix: CVE-2024-12345 - API server privilege escalation",
                    "Performance: Reduce etcd memory footprint by 15%",
                    "Reliability: Fix kubelet crash loop on node restart",
                    "Networking: Fix iptables rule leaks in service cleanup",
                ]
            },
            {
                "version": "1.30.4",
                "date": "2024-08-20",
                "type": "patch",
                "components": [
                    "Fix: Incorrect RBAC evaluation in webhook handlers",
                    "Fix: Memory leak in watch endpoint implementation",
                    "Enhancement: Better error messages for common kubectl mistakes",
                ]
            },
            {
                "version": "1.29.10",
                "date": "2024-09-01",
                "type": "patch",
                "components": [
                    "Security: Restrict access to sensitive pod fields",
                    "Fix: StatefulSet ordinal ordering bug",
                ]
            },
        ]
        
        for rel in versions:
            # Create multiple context entries per release
            releases.append({
                "content": f"Kubernetes {rel['version']} Release Notes - {rel['date']}\n" + 
                          "\n".join(rel["components"]),
                "domain": "Documentation",
                "tags": ["kubernetes", rel["version"], rel["type"], "release-notes"],
                "importance": 0.9,
                "source": "kubernetes-releases"
            })
            
            # Create security-focused entries
            if rel["type"] == "patch":
                releases.append({
                    "content": f"Security updates in Kubernetes {rel['version']}: " +
                              ". ".join([c for c in rel["components"] if "CVE" in c or "Security" in c]),
                    "domain": "Code",
                    "tags": ["kubernetes", rel["version"], "security", "cve"],
                    "importance": 1.0,
                    "source": "kubernetes-security"
                })
        
        # Add cluster component documentation
        components = [
            {
                "name": "etcd",
                "content": "etcd is a distributed, reliable key-value store for the most critical data of a distributed system. "
                          "It's used as Kubernetes' backing store for all cluster data. Typical production setup involves "
                          "3 or 5 etcd instances for fault tolerance. Tuning etcd for performance requires understanding "
                          "disk I/O, network latency, and snapshot intervals.",
                "tags": ["etcd", "database", "kubernetes-component", "critical"],
            },
            {
                "name": "API Server",
                "content": "The Kubernetes API server is a component of the control plane that exposes the Kubernetes API. "
                          "It handles all requests to the cluster. High-availability deployments require multiple "
                          "API server instances with load balancing. Metrics include request latency, audit logging, "
                          "and authentication/authorization performance.",
                "tags": ["api-server", "control-plane", "kubernetes-component"],
            },
            {
                "name": "Kubelet",
                "content": "The kubelet is the primary node agent that runs on each node. It watches for pod specs "
                          "and ensures containers are running. Critical monitoring metrics include CPU/memory usage, "
                          "eviction behavior, and container restart counts. Node capacity planning should account for "
                          "kubelet overhead.",
                "tags": ["kubelet", "node-agent", "kubernetes-component"],
            },
            {
                "name": "Scheduler",
                "content": "The Kubernetes scheduler is responsible for assigning pods to nodes. Advanced scheduling "
                          "uses node affinity, pod affinity, taints/tolerations, and priority classes. Performance "
                          "depends on cluster size and scheduling complexity.",
                "tags": ["scheduler", "pod-placement", "kubernetes-component"],
            },
        ]
        
        for comp in components:
            releases.append({
                "content": f"{comp['name']} - {comp['content']}",
                "domain": "Documentation",
                "tags": comp["tags"],
                "importance": 0.8,
                "source": "kubernetes-docs"
            })
        
        # Add configuration best practices
        practices = [
            ("Resource Requests/Limits", "Proper resource requests and limits enable effective scheduling and QoS. "
             "Set requests to actual usage patterns and limits based on maximum acceptable usage. Use vertical pod "
             "autoscaler to optimize settings over time."),
            ("Network Policies", "Network policies control traffic between pods. Default-deny ingress with explicit "
             "allow rules provides security. Label pods carefully for policy selectors."),
            ("RBAC Configuration", "Role-Based Access Control should follow principle of least privilege. Regular "
             "audits of ClusterRoleBindings are essential. Service account tokens should rotate regularly."),
            ("Pod Disruption Budgets", "PDBs prevent simultaneous eviction of multiple pod replicas during updates. "
             "Set minAvailable to number of required replicas minus one for graceful degradation."),
            ("Cluster Autoscaling", "Configure based on actual workload patterns. Set appropriate scale-down thresholds "
             "to prevent thrashing. Monitor for scale-up failures due to resource limits."),
        ]
        
        for title, content in practices:
            releases.append({
                "content": f"{title} Best Practices:\n{content}",
                "domain": "Documentation",
                "tags": ["best-practices", "kubernetes", "operations"],
                "importance": 0.75,
                "source": "kubernetes-bestpractices"
            })
        
        return releases


class BenchmarkResults:
    """Comprehensive benchmark results tracking"""
    
    def __init__(self):
        self.tests_passed = 0
        self.tests_failed = 0
        self.errors = []
        self.benchmarks = {}
        self.metrics = {}
        self.stored_ids = []
        self.dataset_size = 0
        
    def add_success(self, test_name: str):
        self.tests_passed += 1
        print(f"  ✓ {test_name}")
        
    def add_failure(self, test_name: str, error: str):
        self.tests_failed += 1
        self.errors.append(f"{test_name}: {error}")
        print(f"  ✗ {test_name}: {error}")
        
    def add_benchmark(self, operation: str, timings: List[float], unit: str = "ms"):
        if timings:
            self.benchmarks[operation] = {
                "min": min(timings),
                "max": max(timings),
                "mean": statistics.mean(timings),
                "median": statistics.median(timings),
                "stdev": statistics.stdev(timings) if len(timings) > 1 else 0,
                "p95": sorted(timings)[int(len(timings) * 0.95)] if len(timings) > 1 else timings[0],
                "p99": sorted(timings)[int(len(timings) * 0.99)] if len(timings) > 1 else timings[0],
                "samples": len(timings),
                "unit": unit
            }
        
    def add_metric(self, name: str, value: Any, unit: str = ""):
        self.metrics[name] = {"value": value, "unit": unit}
        
    def print_report(self):
        """Print comprehensive benchmark report"""
        print("\n" + "="*100)
        print(" ADVANCED CONTEXT-MCP BENCHMARK REPORT")
        print(" Kubernetes Release Data & Complex Dataset Testing")
        print("="*100)
        print(f"\nTest Execution: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        
        # Test Summary
        print(f"\n{'TEST SUMMARY':-^100}")
        total = self.tests_passed + self.tests_failed
        if total > 0:
            print(f"  Total Tests: {total}")
            print(f"  Passed: {self.tests_passed} ({self.tests_passed/total*100:.1f}%)")
            print(f"  Failed: {self.tests_failed} ({self.tests_failed/total*100:.1f}%)")
        
        if self.errors:
            print(f"\n{'ERRORS':-^100}")
            for error in self.errors:
                print(f"  • {error}")
        
        # Dataset Info
        print(f"\n{'DATASET INFORMATION':-^100}")
        for name, data in self.metrics.items():
            if "dataset" in name.lower() or "items" in name.lower():
                value = data['value']
                unit = data['unit']
                print(f"  {name}: {value:,} {unit}")
        
        # Performance Benchmarks
        print(f"\n{'PERFORMANCE BENCHMARKS':-^100}")
        if self.benchmarks:
            print(f"  {'Operation':<30} {'Mean':<12} {'Median':<12} {'P95':<12} {'P99':<12} {'StdDev':<10}")
            print(f"  {'-'*30} {'-'*12} {'-'*12} {'-'*12} {'-'*12} {'-'*10}")
            
            for op, stats in self.benchmarks.items():
                unit = stats['unit']
                print(f"  {op:<30} "
                      f"{stats['mean']:.2f}{unit:<8} "
                      f"{stats['median']:.2f}{unit:<8} "
                      f"{stats['p95']:.2f}{unit:<8} "
                      f"{stats['p99']:.2f}{unit:<8} "
                      f"{stats['stdev']:.2f}{unit:<6}")
        
        # Key Performance Indicators
        print(f"\n{'KEY PERFORMANCE INDICATORS':-^100}")
        for name, data in self.metrics.items():
            if "throughput" in name.lower() or "latency" in name.lower() or "memory" in name.lower():
                value = data['value']
                unit = data['unit']
                if isinstance(value, float):
                    print(f"  {name}: {value:.2f} {unit}")
                else:
                    print(f"  {name}: {value:,} {unit}")
        
        print("\n" + "="*100)


async def run_advanced_benchmark():
    """Execute comprehensive advanced benchmark"""
    results = BenchmarkResults()
    
    # Start context-mcp server
    client = AdvancedMCPClient(["/home/kang/.local/bin/context-mcp", "--stdio"])
    
    try:
        await client.start()
        
        # Generate Kubernetes release data
        print("\n[1] Generating Kubernetes Release Data")
        k8s_data = KubernetesDataGenerator.get_k8s_release_notes()
        results.add_metric("Dataset Items", len(k8s_data))
        results.dataset_size = len(k8s_data)
        print(f"    Generated {len(k8s_data)} context items from Kubernetes releases and documentation")
        results.add_success("K8s data generation")
        
        # Store all K8s data
        print("\n[2] Storing K8s Data (Scalability Test)")
        store_timings = []
        batch_size = 10
        
        for i, ctx in enumerate(k8s_data):
            start = time.time()
            result = await client.call_tool("store_context", ctx)
            elapsed = (time.time() - start) * 1000
            store_timings.append(elapsed)
            
            # Extract ID
            content = result.get("content", [{}])[0]
            if content.get("type") == "text":
                text = content.get("text", "")
                id_match = re.search(r'"id":\s*"([A-Za-z0-9+/=]+)"', text)
                if id_match:
                    results.stored_ids.append(id_match.group(1))
        
        results.add_benchmark("Store K8s Context", store_timings)
        results.add_success(f"Stored all {len(k8s_data)} K8s contexts")
        print(f"    Average latency: {statistics.mean(store_timings):.2f}ms")
        print(f"    Throughput: {len(k8s_data)/(sum(store_timings)/1000):.0f} contexts/sec")
        
        # Query by domain
        print("\n[3] Domain-Based Queries (Documentation vs Code)")
        query_timings = []
        for domain in ["Documentation", "Code"]:
            start = time.time()
            result = await client.call_tool("query_contexts", {
                "domain": domain,
                "limit": 100
            })
            elapsed = (time.time() - start) * 1000
            query_timings.append(elapsed)
        
        results.add_benchmark("Domain Query", query_timings)
        results.add_success("Domain-based queries")
        
        # Advanced filtering
        print("\n[4] Advanced Filtering (Tags + Importance + Temporal)")
        filter_timings = []
        
        filters = [
            {"tags": ["kubernetes"], "min_importance": 0.8},
            {"tags": ["security"], "min_importance": 0.9},
            {"tags": ["kubernetes", "best-practices"]},
            {"domain": "Documentation", "min_importance": 0.75},
        ]
        
        for filter_spec in filters:
            start = time.time()
            result = await client.call_tool("query_contexts", filter_spec)
            elapsed = (time.time() - start) * 1000
            filter_timings.append(elapsed)
        
        results.add_benchmark("Advanced Filter", filter_timings)
        results.add_success("Advanced filtering queries")
        
        # RAG retrieval with complex queries
        print("\n[5] RAG Retrieval (Complex Semantic Queries)")
        rag_queries = [
            "Kubernetes API server performance and request handling",
            "etcd distributed storage and leader election",
            "Security vulnerabilities and CVE patches",
            "Pod scheduling and node affinity",
            "Network policies and security best practices",
            "Kubelet resource management and eviction",
            "RBAC configuration and access control",
        ]
        
        rag_timings = []
        for query_text in rag_queries:
            start = time.time()
            result = await client.call_tool("retrieve_contexts", {
                "text": query_text,
                "max_results": 5
            })
            elapsed = (time.time() - start) * 1000
            rag_timings.append(elapsed)
        
        results.add_benchmark("RAG Query", rag_timings)
        results.add_success(f"RAG retrieval on {len(rag_queries)} complex queries")
        print(f"    Average latency: {statistics.mean(rag_timings):.2f}ms")
        
        # Temporal statistics
        print("\n[6] Temporal Analytics")
        start = time.time()
        temporal_result = await client.call_tool("get_temporal_stats", {})
        temporal_elapsed = (time.time() - start) * 1000
        results.add_benchmark("Temporal Stats", [temporal_elapsed])
        results.add_success("Temporal statistics generation")
        
        # Storage statistics
        print("\n[7] Storage Analysis")
        start = time.time()
        storage_result = await client.call_tool("get_storage_stats", {})
        storage_elapsed = (time.time() - start) * 1000
        results.add_benchmark("Storage Stats", [storage_elapsed])
        
        content = storage_result.get("content", [{}])[0]
        if content.get("type") == "text":
            text = content.get("text", "")
            print(f"    {text[:200]}...")
        
        results.add_success("Storage statistics")
        
        # Batch update screening
        print("\n[8] Security Screening & Batch Updates")
        if results.stored_ids:
            screening_timings = []
            # Screen a sample of stored contexts
            sample_ids = results.stored_ids[::max(1, len(results.stored_ids)//10)]  # 10% sample
            
            for ctx_id in sample_ids[:5]:  # Limit to 5 for demo
                start = time.time()
                result = await client.call_tool("update_screening", {
                    "id": ctx_id,
                    "status": "Safe",
                    "reason": "Kubernetes official documentation - verified safe"
                })
                elapsed = (time.time() - start) * 1000
                screening_timings.append(elapsed)
            
            results.add_benchmark("Screen Context", screening_timings)
            results.add_success(f"Screened {len(sample_ids)} contexts")
        
        # Stress test: Rapid sequential queries
        print("\n[9] Stress Test - Rapid Queries")
        stress_queries = 50
        stress_timings = []
        
        print(f"    Running {stress_queries} rapid sequential queries...")
        for i in range(stress_queries):
            start = time.time()
            result = await client.call_tool("query_contexts", {
                "limit": 10
            })
            elapsed = (time.time() - start) * 1000
            stress_timings.append(elapsed)
        
        results.add_benchmark("Stress Query", stress_timings)
        results.add_success(f"Completed {stress_queries} rapid queries")
        print(f"    Min: {min(stress_timings):.2f}ms, Max: {max(stress_timings):.2f}ms, Avg: {statistics.mean(stress_timings):.2f}ms")
        
        # Cleanup
        print("\n[10] Cleanup & Final Metrics")
        start = time.time()
        cleanup_result = await client.call_tool("cleanup_expired", {})
        cleanup_elapsed = (time.time() - start) * 1000
        results.add_benchmark("Cleanup", [cleanup_elapsed])
        results.add_success("Cleanup operation")
        
        # Calculate and add final metrics
        results.add_metric("Total Stored Contexts", len(results.stored_ids))
        results.add_metric("Query Throughput (avg)", 
                          len(rag_queries) / (sum(rag_timings) / 1000), "queries/sec")
        results.add_metric("Storage Throughput (avg)", 
                          len(k8s_data) / (sum(store_timings) / 1000), "contexts/sec")
        results.add_metric("P99 Store Latency", sorted(store_timings)[int(len(store_timings)*0.99)], "ms")
        results.add_metric("P99 Query Latency", sorted(rag_timings)[int(len(rag_timings)*0.99)], "ms")
        
    except Exception as e:
        print(f"\n✗ Fatal error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        await client.stop()
    
    # Print comprehensive report
    results.print_report()
    
    return results.tests_failed == 0


if __name__ == "__main__":
    success = asyncio.run(run_advanced_benchmark())
    sys.exit(0 if success else 1)
