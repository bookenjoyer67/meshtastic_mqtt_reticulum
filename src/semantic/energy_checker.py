"""
Energy Checker Module

This module provides energy level tracking and checking for different phases
of program execution. It defines energy levels (low, medium, high, critical)
and associates default energy levels with different execution phases.
"""

from enum import Enum, auto
from typing import Dict, Optional, Union
from dataclasses import dataclass


class EnergyLevel(Enum):
    """Energy levels for program execution phases."""
    LOW = auto()
    MEDIUM = auto()
    HIGH = auto()
    CRITICAL = auto()
    
    def __lt__(self, other: 'EnergyLevel') -> bool:
        """Compare energy levels for ordering."""
        if not isinstance(other, EnergyLevel):
            return NotImplemented
        return self.value < other.value
    
    def __le__(self, other: 'EnergyLevel') -> bool:
        """Compare energy levels for ordering."""
        if not isinstance(other, EnergyLevel):
            return NotImplemented
        return self.value <= other.value
    
    def __gt__(self, other: 'EnergyLevel') -> bool:
        """Compare energy levels for ordering."""
        if not isinstance(other, EnergyLevel):
            return NotImplemented
        return self.value > other.value
    
    def __ge__(self, other: 'EnergyLevel') -> bool:
        """Compare energy levels for ordering."""
        if not isinstance(other, EnergyLevel):
            return NotImplemented
        return self.value >= other.value
    
    def __str__(self) -> str:
        """String representation of energy level."""
        return self.name.lower()


class Phase(Enum):
    """Execution phases for energy tracking."""
    INITIALIZATION = "initialization"
    COMPUTATION = "computation"
    IO = "io"
    CLEANUP = "cleanup"
    
    def __str__(self) -> str:
        """String representation of phase."""
        return self.value


@dataclass
class EnergyCheck:
    """Result of an energy check."""
    phase: Phase
    expected: EnergyLevel
    actual: EnergyLevel
    is_valid: bool
    
    def __str__(self) -> str:
        """String representation of energy check result."""
        status = "✓" if self.is_valid else "✗"
        return f"{status} {self.phase}: expected {self.expected}, got {self.actual}"


class EnergyChecker:
    """
    Energy checker for tracking and validating energy levels during execution.
    
    This class maintains energy level expectations for different phases
    and can validate actual energy usage against those expectations.
    """
    
    # Default energy levels for each phase
    # Note: IO is set to HIGH as it typically involves disk/network operations
    # which are more energy-intensive than computation
    DEFAULT_PHASE_ENERGY: Dict[Phase, EnergyLevel] = {
        Phase.INITIALIZATION: EnergyLevel.LOW,
        Phase.COMPUTATION: EnergyLevel.MEDIUM,
        Phase.IO: EnergyLevel.HIGH,
        Phase.CLEANUP: EnergyLevel.LOW,
    }
    
    def __init__(self, custom_defaults: Optional[Dict[Phase, EnergyLevel]] = None):
        """
        Initialize energy checker with optional custom defaults.
        
        Args:
            custom_defaults: Optional dictionary mapping phases to custom
                            energy level defaults. If None, uses the built-in
                            defaults.
        """
        self.phase_energy = self.DEFAULT_PHASE_ENERGY.copy()
        if custom_defaults:
            self.phase_energy.update(custom_defaults)
        
        # Track current phase and energy level
        self.current_phase: Optional[Phase] = None
        self.current_energy: Optional[EnergyLevel] = None
    
    def set_phase(self, phase: Union[Phase, str]) -> None:
        """
        Set the current execution phase.
        
        Args:
            phase: Phase enum or string name of phase
        """
        if isinstance(phase, str):
            try:
                phase = Phase(phase)
            except ValueError:
                raise ValueError(f"Invalid phase: {phase}. Valid phases are: {[p.value for p in Phase]}")
        
        self.current_phase = phase
        self.current_energy = None
    
    def set_energy(self, energy: Union[EnergyLevel, str]) -> None:
        """
        Set the current energy level.
        
        Args:
            energy: EnergyLevel enum or string name of energy level
        """
        if isinstance(energy, str):
            try:
                energy = EnergyLevel[energy.upper()]
            except KeyError:
                raise ValueError(f"Invalid energy level: {energy}. Valid levels are: {[e.name.lower() for e in EnergyLevel]}")
        
        self.current_energy = energy
    
    def check(self, phase: Optional[Union[Phase, str]] = None, 
              energy: Optional[Union[EnergyLevel, str]] = None) -> EnergyCheck:
        """
        Check if energy level is appropriate for the given phase.
        
        Args:
            phase: Phase to check (uses current phase if not specified)
            energy: Energy level to check (uses current energy if not specified)
            
        Returns:
            EnergyCheck object with validation result
            
        Raises:
            ValueError: If no phase or energy is specified and none is set
        """
        # Determine phase to check
        if phase is None:
            if self.current_phase is None:
                raise ValueError("No phase specified and no current phase set")
            check_phase = self.current_phase
        else:
            if isinstance(phase, str):
                check_phase = Phase(phase)
            else:
                check_phase = phase
        
        # Determine energy to check
        if energy is None:
            if self.current_energy is None:
                raise ValueError("No energy specified and no current energy set")
            check_energy = self.current_energy
        else:
            if isinstance(energy, str):
                check_energy = EnergyLevel[energy.upper()]
            else:
                check_energy = energy
        
        # Get expected energy for this phase
        expected_energy = self.phase_energy.get(check_phase)
        if expected_energy is None:
            # If phase not in defaults, assume MEDIUM
            expected_energy = EnergyLevel.MEDIUM
        
        # Check if energy is valid (actual <= expected)
        is_valid = check_energy <= expected_energy
        
        return EnergyCheck(
            phase=check_phase,
            expected=expected_energy,
            actual=check_energy,
            is_valid=is_valid
        )
    
    def get_expected_energy(self, phase: Union[Phase, str]) -> EnergyLevel:
        """
        Get the expected energy level for a given phase.
        
        Args:
            phase: Phase to get expected energy for
            
        Returns:
            Expected energy level for the phase
        """
        if isinstance(phase, str):
            phase = Phase(phase)
        
        expected = self.phase_energy.get(phase)
        if expected is None:
            # If phase not in defaults, assume MEDIUM
            expected = EnergyLevel.MEDIUM
        
        return expected
    
    def set_custom_default(self, phase: Union[Phase, str], energy: Union[EnergyLevel, str]) -> None:
        """
        Set a custom default energy level for a phase.
        
        Args:
            phase: Phase to set custom default for
            energy: Energy level to set as default
        """
        if isinstance(phase, str):
            phase = Phase(phase)
        
        if isinstance(energy, str):
            energy = EnergyLevel[energy.upper()]
        
        self.phase_energy[phase] = energy
    
    def reset_defaults(self) -> None:
        """Reset all phase energy defaults to built-in defaults."""
        self.phase_energy = self.DEFAULT_PHASE_ENERGY.copy()
    
    def __str__(self) -> str:
        """String representation of energy checker state."""
        lines = ["EnergyChecker:"]
        lines.append(f"  Current phase: {self.current_phase}")
        lines.append(f"  Current energy: {self.current_energy}")
        lines.append("  Phase defaults:")
        for phase, energy in sorted(self.phase_energy.items(), key=lambda x: str(x[0])):
            lines.append(f"    {phase}: {energy}")
        return "\n".join(lines)


# Convenience functions for common operations
def create_default_checker() -> EnergyChecker:
    """Create an energy checker with default settings."""
    return EnergyChecker()


def check_energy_for_phase(phase: Union[Phase, str], energy: Union[EnergyLevel, str]) -> EnergyCheck:
    """
    Quick check of energy level for a phase using default settings.
    
    Args:
        phase: Phase to check
        energy: Energy level to check
        
    Returns:
        EnergyCheck object with validation result
    """
    checker = EnergyChecker()
    return checker.check(phase, energy)


def get_all_phase_defaults() -> Dict[Phase, EnergyLevel]:
    """
    Get all default phase energy mappings.
    
    Returns:
        Dictionary mapping all phases to their default energy levels
    """
    return EnergyChecker.DEFAULT_PHASE_ENERGY.copy()


if __name__ == "__main__":
    # Example usage
    print("=== Energy Checker Example ===\n")
    
    # Create a checker with defaults
    checker = EnergyChecker()
    print("Default configuration:")
    print(checker)
    print()
    
    # Check some examples
    print("Example checks:")
    
    # Valid: LOW energy for INITIALIZATION
    result = checker.check(Phase.INITIALIZATION, EnergyLevel.LOW)
    print(f"  {result}")
    
    # Invalid: HIGH energy for INITIALIZATION (should be LOW)
    result = checker.check(Phase.INITIALIZATION, EnergyLevel.HIGH)
    print(f"  {result}")
    
    # Valid: MEDIUM energy for COMPUTATION
    result = checker.check("computation", "medium")
    print(f"  {result}")
    
    # Valid: HIGH energy for IO
    result = checker.check("io", "high")
    print(f"  {result}")
    
    # Invalid: CRITICAL energy for CLEANUP (should be LOW)
    result = checker.check("cleanup", "critical")
    print(f"  {result}")
    
    print("\n=== Custom Configuration Example ===\n")
    
    # Create checker with custom defaults
    custom_defaults = {
        Phase.IO: EnergyLevel.MEDIUM,  # Set IO to MEDIUM instead of HIGH
        Phase("computation"): EnergyLevel.HIGH,  # Set computation to HIGH
    }
    custom_checker = EnergyChecker(custom_defaults)
    print("Custom configuration (IO=MEDIUM, COMPUTATION=HIGH):")
    print(custom_checker)
    print()
    
    # Now HIGH energy for COMPUTATION is valid
    result = custom_checker.check("computation", "high")
    print(f"  {result}")
    
    # And MEDIUM energy for IO is valid
    result = custom_checker.check("io", "medium")
    print(f"  {result}")