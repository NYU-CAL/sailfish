#define BETA_TVD 1.0
#define NPOLY 3      // Hard wire for 1D 3rd order for now
#define NUM_POINTS 3 // Hard wire for 1D 3rd order for now
#define PDE 0        // 0 for linear advection, 1 for Burgers
#define WAVESPEED 1.0

PRIVATE double flux(double ux) 
{
    switch (PDE) {
        case 0: return WAVESPEED * ux; // Advection
        case 1: return 0.5 * ux * ux;  // Burgers
    }
}

PRIVATE double upwind(double ul, double ur) 
{
    switch (PDE) {
        case 0: // advection
            if (WAVESPEED > 0.0){
                return flux(ul);
            }
            else{
                return flux(ur);
            }
        case 1: // Burgers
            double al = ul;
            double ar = ur;
            if (al > 0.0 && ar > 0.0){
                return flux(ul);
            }
            else if (al < 0.0 && ar < 0.0){
                return flux(ur);
            }
            else {
                return 0.0;
            } 
    }
}

PRIVATE double dot(double *u, double *p) 
{
    double sum = 0.0;
    for (int i = 0, i < NPOLY, ++i){
        sum += u[i] * p[i]; 
    }
    return sum;
}

"""
Advance solution one Runge-Kutta substep
"""
PUBLIC void scdg_1d_advance_rk(
    int num_zones,    // number of zones, not including guard zones
    double *u_rk,     // :: $.shape == (num_zones + 2, NPOLY)
    double *u_rd,     // :: $.shape == (num_zones + 2, NPOLY)
    double *u_wr,     // :: $.shape == (num_zones + 2, NPOLY)
    double time,      // current time
    double rk_param,  // Runge-Kutta parameter
    double dt)        // time step
{
    int ng = 1; // number of guard zones

    // TODO: pass cell data as a struct argument 

    // Gaussian quadrature points in scaled domain xsi=[-1,1]
    double g =  {-0.774596669241483, 0.000000000000000, 0.774596669241483}; 
    // Gaussian weights at quadrature points
    double w =  { 0.555555555555556, 0.888888888888889, 0.555555555555556};
    // Scaled LeGendre polynomials at quadrature points
    double p = {{ 1.000000000000000, 1.000000000000000, 1.000000000000000},
                {-1.341640786499873, 0.000000000000000, 1.341640786499873},
                { 0.894427190999914, -1.11803398874990, 0.894427190999914}}; 
    // Derivative of Scaled LeGendre polynomials at quadrature points
    double pp = {{ 0.000000000000000, 0.000000000000000, 0.000000000000000},
                 { 1.732050807568877, 1.732050807568877, 1.732050807568877},
                 {-5.196152422706629, 0.000000000000000, 5.196152422706629}};
    // Unit normal vector at left and right faces
    double nhat = {-1.0, 1.0};
    // Scaled LeGendre polynomials at left face
    double pfl = {1.000000000000000, -1.732050807568877, 2.23606797749979};
    // Scaled LeGendre polynomials at right face
    double pfr = {1.000000000000000,  1.732050807568877, 2.23606797749979};    

    FOR_EACH_1D(num_zones)
    {
        double *urd = &conserved_rd[NPOLY * (i + ng)];
        double *uli = &conserved_rd[NPOLY * (i + ng - 1)];
        double *uri = &conserved_rd[NPOLY * (i + ng + 1)];
        double *uwr = &conserved_wr[NPOLY * (i + ng)];
        double *urk = &conserved_rk[NPOLY * (i + ng)];

        double uimh_l = dot(uli, pfr);
        double uimh_r = dot(urd, pfl);
        double uiph_l = dot(urd, pfr);
        double uiph_r = dot(uri, pfl);
        double fimh = upwind(uimh_l, uimh_r);
        double fiph = upwind(uiph_l, uiph_r);

        double fx[NUM_POINTS];

        for (int n = 0; n < NUM_POINTS; ++n)
        {
            double ux = 0.0;
            for (int l = 0; l < NPOLY; ++l)
            {
                ux += urd[l] * p[l, n];
            }
            fx[n] = flux(ux); 
        }

        for (int l = 0; l < NPOLY; ++l)
        {
            double udot_v = 0.0;
            for (int n = 0; n < NUM_POINTS; ++n)
            {
                udot_v += fx[n] * pp[l, n] * w[n] / dx;
            }
            double udot_s = -(fimh * pfl[l] * nhat[0] + fiph * pfr[l] * nhat[1]) / dx;
            uwr[l] = urd[l] + (udot_v + udot_s) * dt;
            uwr[l] = (1.0 - rk_param) * uwr[l] + rk_param * urk[l];
        }
    }
}
