/**
 * WebSocket client for game state updates
 * Handles connection management and auto-reconnection
 */

// This file is intentionally minimal because we're using HTMX's built-in WebSocket extension
// The extension handles most of the heavy lifting for us via ws-connect attribute

// Add connection status indicator
function addConnectionStatus() {
    const navbar = document.querySelector('.navbar');
    if (!navbar) return;

    const statusDiv = document.createElement('div');
    statusDiv.id = 'ws-status';
    statusDiv.className = 'badge badge-sm hidden';
    navbar.appendChild(statusDiv);
}

// Show connection status
function showConnectionStatus(status, message) {
    const statusDiv = document.getElementById('ws-status');
    if (!statusDiv) return;

    statusDiv.classList.remove('hidden', 'badge-success', 'badge-error', 'badge-warning');

    if (status === 'connected') {
        statusDiv.classList.add('badge-success');
        statusDiv.textContent = '● Connected';
    } else if (status === 'disconnected') {
        statusDiv.classList.add('badge-error');
        statusDiv.textContent = '● Disconnected';
    } else if (status === 'reconnecting') {
        statusDiv.classList.add('badge-warning');
        statusDiv.textContent = '● Reconnecting...';
    }
}

// Listen for HTMX WebSocket events
document.addEventListener('htmx:wsOpen', function(event) {
    console.log('WebSocket connected');
    showConnectionStatus('connected');
});

document.addEventListener('htmx:wsClose', function(event) {
    console.log('WebSocket disconnected');
    showConnectionStatus('disconnected');
    // HTMX will handle auto-reconnection
});

document.addEventListener('htmx:wsError', function(event) {
    console.error('WebSocket error:', event.detail);
    showConnectionStatus('reconnecting');
});

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    addConnectionStatus();
});
