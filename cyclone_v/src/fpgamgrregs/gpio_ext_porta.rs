use bitfield::bitfield;

bitfield! {
    /// Reading this register reads the values of the GPIO inputs.
    pub struct GpioExtPortA(u32);
    impl Debug;

/// Reading this provides the value of FPGA_POWER_ON
    fpo, _: 11;

    /// Reading this provides the value of CONF_DONE Pin
    pub cdp, _: 10;

    /// Reading this provides the value of nSTATUS Pin
    pub nsp, _: 9;

    /// Reading this provides the value of nCONFIG Pin
    pub ncp, _: 8;

    /// Reading this provides the value of PR_DONE
    pub prd, _: 7;

    /// Reading this provides the value of PR_ERROR
    pub pre, _: 6;

    /// Reading this provides the value of PR_READY
    pub prr, _: 5;

    /// Reading this provides the value of CVP_CONF_DONE
    pub ccd, _: 4;

    /// Reading this provides the value of CRC_ERROR
    pub crc, _: 3;

    /// Reading this provides the value of INIT_DONE
    pub id, _: 2;

    /// Reading this provides the value of CONF_DONE.
    pub cd, _: 1;

    /// Reading this provides the value of nSTATUS.
    pub ns, _: 0;
}
